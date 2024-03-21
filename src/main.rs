use axum::{
    Router, serve,
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post}
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::{
    net::SocketAddr,
    sync::Arc
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

mod app;
mod config;
mod core;
mod db;
mod errors;
mod extractors;
mod handlers;
mod jwt;
mod model;
mod module;
mod pagination;
mod params;
mod prod_core;
mod sqlite;
mod time;
mod upload;
mod version;

use crate::{
    app::AppState,
    config::Config,
    core::CoreArc,
    prod_core::ProdCore,
    errors::AppError,
    jwt::DecodingKey,
    sqlite::SqlxDatabaseClient,
    upload::LocalUploader,
};

impl From<&AppError> for StatusCode {
    fn from(err: &AppError) -> Self {
        match err {
            AppError::BadMimeType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppError::CannotRemoveLastOwner => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonError => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::LimitOutOfRange => StatusCode::BAD_REQUEST,
            AppError::MalformedQuery => StatusCode::BAD_REQUEST,
            AppError::MalformedVersion => StatusCode::BAD_REQUEST,
            AppError::NotAPackage => StatusCode::NOT_FOUND,
            AppError::NotAProject => StatusCode::NOT_FOUND,
            AppError::NotARevision => StatusCode::NOT_FOUND,
            AppError::NotAUser => StatusCode::NOT_FOUND,
            AppError::NotAVersion => StatusCode::NOT_FOUND,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct HttpError {
    error: String
}

impl From<AppError> for HttpError {
    fn from(err: AppError) -> Self {
        HttpError { error: format!("{}", err) }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = StatusCode::from(&self);
        let body = Json(HttpError::from(self));
        (code, body).into_response()
    }
}

fn routes(api: &str) -> Router<AppState> {
    Router::new()
        .route(
            &format!("{api}/"),
            get(handlers::root_get)
        )
        .route(
            &format!("{api}/projects"),
            get(handlers::projects_get)
        )
        .route(
            &format!("{api}/projects/:proj"),
            get(handlers::project_get)
            .post(handlers::project_post)
            .patch(handlers::project_patch)
        )
        .route(
            &format!("{api}/projects/:proj/:revision"),
            get(handlers::project_revision_get)
        )
        .route(
            &format!("{api}/projects/:proj/owners"),
            get(handlers::owners_get)
            .put(handlers::owners_add)
            .delete(handlers::owners_remove)
        )
        .route(
            &format!("{api}/projects/:proj/players"),
            get(handlers::players_get)
            .put(handlers::players_add)
            .delete(handlers::players_remove)
        )
        .route(
            &format!("{api}/projects/:proj/packages/:pkg_name"),
            get(handlers::release_get)
            .post(handlers::packages_post)
        )
        .route(
            &format!("{api}/projects/:proj/packages/:pkg_name/:version"),
            get(handlers::release_version_get)
            .put(handlers::release_put)
        )
        .route(
            &format!("{api}/projects/:proj/images/:img_name"),
            get(handlers::image_get)
            .post(handlers::image_post)
        )
        .route(
            &format!("{api}/projects/:proj/images/:img_name/:revision"),
            get(handlers::image_revision_get)
        )
        .route(
            &format!("{api}/projects/:proj/flag"),
            post(handlers::flag_post)
        )
        .fallback(handlers::not_found)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::very_permissive())
        )
}

#[tokio::main]
async fn main() {
    let config = Config {
        db_path: "projects.db".into(),
// TODO: read key from file? env?
        jwt_key: b"@wlD+3L)EHdv28u)OFWx@83_*TxhVf9IdUncaAz6ICbM~)j+dH=sR2^LXp(tW31z".to_vec(),
        api_base_path: "/api/v1".into(),
        listen_ip: [0, 0, 0, 0],
        listen_port: 3000
    };

// TODO: handle error?
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite://{}", &config.db_path))
        .await
        .unwrap();

    let core = ProdCore {
        db: SqlxDatabaseClient(db_pool),
        uploader: LocalUploader { uploads_directory: "uploads".into() },
        now: Utc::now
    };

    let state = AppState {
        key: DecodingKey::from_secret(&config.jwt_key),
        core: Arc::new(core) as CoreArc
    };

    let api = &config.api_base_path;

    let app: Router = routes(api)
        .with_state(state);

    let addr = SocketAddr::from((config.listen_ip, config.listen_port));
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app)
        .await
        .unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    use axum::{
        body::{self, Body, Bytes},
        http::{
            Method, Request,
            header::{AUTHORIZATION, CONTENT_TYPE, LOCATION}
        }
    };
    use mime::{APPLICATION_JSON, TEXT_PLAIN};
    use once_cell::sync::Lazy;
    use tower::ServiceExt; // for oneshot

    use crate::{
        core::{Core, CoreError},
        jwt::{self, EncodingKey},
        model::{GameData, Owner, PackageData, Package, ProjectData, ProjectDataPatch, ProjectDataPost, Project, Projects, ProjectSummary, ReleaseData, User, Users},
        pagination::{Anchor, Direction, Limit, SortBy, Pagination, Seek, SeekLink},
        params::ProjectsParams,
        version::Version
    };

    const API_V1: &str = "/api/v1";
    const KEY: &[u8] = b"@wlD+3L)EHdv28u)OFWx@83_*TxhVf9IdUncaAz6ICbM~)j+dH=sR2^LXp(tW31z";

    async fn body_bytes(r: Response) -> Bytes {
        body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    }

    async fn body_as<D: for<'a> Deserialize<'a>>(r: Response) -> D {
        serde_json::from_slice::<D>(&body_bytes(r).await).unwrap()
    }

    async fn body_empty(r: Response) -> bool {
        body_bytes(r).await.is_empty()
    }

// TODO: fill in the fields
// TODO: can these be declared some other way?
    static PROJECT_SUMMARY_A: Lazy<ProjectSummary> = Lazy::new(|| {
        ProjectSummary {
            name: "project_a".into(),
            description: "".into(),
            revision: 1,
            created_at: "".into(),
            modified_at: "".into(),
            tags: vec![],
            game: GameData {
                title: "".into(),
                title_sort_key: "".into(),
                publisher: "".into(),
                year: "".into()
            }
        }
    });

    static PROJECT_SUMMARY_B: Lazy<ProjectSummary> = Lazy::new(|| {
        ProjectSummary {
            name: "project_b".into(),
            description: "".into(),
            revision: 1,
            created_at: "".into(),
            modified_at: "".into(),
            tags: vec![],
            game: GameData {
                title: "".into(),
                title_sort_key: "".into(),
                publisher: "".into(),
                year: "".into()
            }
        }
    });

    const BOB_UID: i64 = 1;

    #[derive(Clone)]
    struct TestCore { }

    #[axum::async_trait]
    impl Core for TestCore {
        async fn get_project_id(
            &self,
            proj: &str,
        ) -> Result<Project, CoreError>
        {
            match proj {
                "a_project" => Ok(Project(1)),
                _ => Err(CoreError::NotAProject)
            }
        }

        async fn get_package_id(
            &self,
            _proj: Project,
            pkg: &str
        ) -> Result<Package, CoreError>
        {
            match pkg {
                "a_package" => Ok(Package(1)),
                _ => Err(CoreError::NotAPackage)
            }
        }

        async fn user_is_owner(
            &self,
            user: User,
            _proj: Project
        ) -> Result<bool, CoreError>
        {
            Ok(user == User(1) || user == User(2))
        }

        async fn add_owners(
            &self,
            _owners: &Users,
            _proj: Project
        ) -> Result<(), CoreError>
        {
            Ok(())
        }

        async fn remove_owners(
            &self,
            _owners: &Users,
            _proj: Project
        ) -> Result<(), CoreError>
        {
            Ok(())
        }

        async fn get_owners(
            &self,
            _proj: Project
        ) -> Result<Users, CoreError>
        {
            Ok(
                Users {
                    users: vec![
                        "alice".into(),
                        "bob".into()
                    ]
                }
            )
        }

        async fn get_projects(
            &self,
            params: ProjectsParams
        ) -> Result<Projects, CoreError>
        {
            Ok(
                Projects {
                    projects: vec![
                        PROJECT_SUMMARY_A.clone(),
                        PROJECT_SUMMARY_B.clone()
                    ],
                    meta: Pagination {
                        prev_page: Some(
                            SeekLink::new(
                                &Seek {
                                    anchor: Anchor::Before("project_a".into(), 0),
                                    sort_by: SortBy::ProjectName,
                                    dir: Direction::Ascending
                                },
                                params.limit
                            ).unwrap()
                        ),
                        next_page: Some(
                            SeekLink::new(
                                &Seek {
                                    anchor: Anchor::After("project_b".into(), 0),
                                    sort_by: SortBy::ProjectName,
                                    dir: Direction::Ascending
                                },
                                params.limit
                            ).unwrap()
                        ),
                        total: 1234
                    }
                }
            )
        }

        async fn get_project(
            &self,
            _proj: Project,
        ) -> Result<ProjectData, CoreError>
        {
            Ok(
                ProjectData {
                    name: "eia".into(),
                    description: "A module for Empires in Arms".into(),
                    revision: 1,
                    created_at: "2023-10-26T00:00:00,000000000+01:00".into(),
                    modified_at: "2023-10-30T18:53:53,056386142+00:00".into(),
                    tags: vec![],
                    game: GameData {
                        title: "Empires in Arms".into(),
                        title_sort_key: "Empires in Arms".into(),
                        publisher: "Avalon Hill".into(),
                        year: "1983".into()
                    },
                    readme: "".into(),
                    image: None,
                    owners: vec!["alice".into(), "bob".into()],
                    packages: vec![
                        PackageData {
                            name: "".into(),
                            description: "".into(),
                            releases: vec![
                                ReleaseData {
                                    version: "".into(),
                                    filename: "".into(),
                                    url: "".into(),
                                    size: 0,
                                    checksum: "".into(),
                                    published_at: "".into(),
                                    published_by: "".into(),
                                    requires: "".into(),
                                    authors: vec![]
                                }
                            ]
                        }
                    ]
                }
            )
        }

        async fn create_project(
            &self,
            _user: User,
            _proj: &str,
            _proj_data: &ProjectDataPost
        ) -> Result<(), CoreError>
        {
            Ok(())
        }

        async fn update_project(
            &self,
            _owner: Owner,
            _proj: Project,
            _proj_data: &ProjectDataPatch
        ) -> Result<(), CoreError>
        {
            Ok(())
        }

        async fn get_project_revision(
            &self,
            proj: Project,
            revision: i64
        ) -> Result<ProjectData, CoreError>
        {
            match revision {
                1 => self.get_project(proj).await,
                _ => Err(CoreError::NotARevision)
            }
        }

        async fn get_release(
            &self,
            _proj: Project,
            _pkg: Package
        ) -> Result<String, CoreError>
        {
            Ok("https://example.com/package".into())
        }

        async fn get_release_version(
            &self,
            _proj: Project,
            _pkg: Package,
            version: &Version
        ) -> Result<String, CoreError>
        {
            match version {
                Version { major: 1, minor: 2, patch: 3, .. } => {
                    Ok("https://example.com/package-1.2.3".into())
                },
                _ => Err(CoreError::NotAVersion)
            }
        }

        async fn get_players(
            &self,
            _proj: Project
        ) -> Result<Users, CoreError>
        {
            Ok(
                Users {
                    users: vec![
                        "player 1".into(),
                        "player 2".into()
                    ]
                }
            )
        }

        async fn add_player(
            &self,
            _player: User,
            _proj: Project
        ) -> Result<(), CoreError>
        {
            Ok(())
        }

        async fn remove_player(
            &self,
            _player: User,
            _proj: Project
        ) -> Result<(), CoreError>
        {
            Ok(())
        }

        async fn get_image(
            &self,
            proj: Project,
            img_name: &str
        ) -> Result<String, CoreError>
        {
            if proj == Project(1) && img_name == "img.png" {
                Ok("https://example.com/img.png".into())
            }
            else {
                Err(CoreError::NotFound)
            }
        }
    }

    fn test_state() -> AppState {
        AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as CoreArc
        }
    }

    fn token(uid: i64) -> String {
        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, uid, 899999999999).unwrap();
        format!("Bearer {token}")
    }

    async fn try_request(request: Request<Body>) -> Response {
        routes(API_V1)
            .with_state(test_state())
            .oneshot(request)
            .await
            .unwrap()
    }

    fn headers<'a>(
        response: &'a Response,
        header_name: &str
    ) -> Vec<&'a [u8]>
    {
        let mut values = response
            .headers()
            .get_all(header_name)
            .iter()
            .flat_map(|v| v.as_ref().split(|b| b == &b','))
            .map(|v| if v[0] == b' ' { &v[1..] } else { v })
            .collect::<Vec<_>>();

        values.sort();
        values
    }

    #[tokio::test]
    async fn cors_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            headers(&response, "access-control-allow-credentials"),
            ["true".as_bytes()]
        );

        assert_eq!(
            headers(&response, "vary"),
            [
                "access-control-request-headers".as_bytes(),
                "access-control-request-method".as_bytes(),
                "origin".as_bytes()
            ]
        );
    }

    #[tokio::test]
    async fn root_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(&body_bytes(response).await[..], b"hello world");
    }

    #[tokio::test]
    async fn get_projects_no_params_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending

                            },
                            None
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_limit_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?limit=5"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            Limit::new(5)
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            Limit::new(5)
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_limit_zero() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?limit=0"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::LimitOutOfRange)
        );
    }

    #[tokio::test]
    async fn get_projects_limit_too_large() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?limit=100000"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::LimitOutOfRange)
        );
    }

    #[tokio::test]
    async fn get_projects_limit_empty() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?limit="))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::LimitOutOfRange)
        );
    }

    #[tokio::test]
    async fn get_projects_limit_not_a_number() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?limit=eleventeen"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::LimitOutOfRange)
        );
    }

    #[tokio::test]
    async fn get_projects_seek_start_ok() {
        let query = SeekLink::new(
            &Seek {
                anchor: Anchor::Start,
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            },
            None
        ).unwrap();

        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects{query}"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_seek_end_ok() {
        let query = SeekLink::new(
            &Seek {
                anchor: Anchor::Start,
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending
            },
            None
        ).unwrap();

        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects{query}"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_seek_before_ok() {
        let query = SeekLink::new(
            &Seek {
                anchor: Anchor::Before("xyz".into(), 0),
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            },
            None
        ).unwrap();

        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects{query}"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_seek_after_ok() {
        let query = SeekLink::new(
            &Seek {
                anchor: Anchor::After("xyz".into(), 0),
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            },
            None
        ).unwrap();

        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects{query}"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            None
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_seek_empty() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?seek="))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::MalformedQuery)
        );
    }

    #[tokio::test]
    async fn get_projects_seek_bad() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects?seek=%@$"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::MalformedQuery)
        );
    }

// TODO: seek string too long?

    #[tokio::test]
    async fn get_projects_seek_and_limit_ok() {
        let query = SeekLink::new(
            &Seek {
                anchor: Anchor::Start,
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            },
            Limit::new(5)
        ).unwrap();

        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects{query}"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            Limit::new(5)
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            Limit::new(5)
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_projects_limit_and_seek_ok() {
        let query = SeekLink::new(
            &Seek {
                anchor: Anchor::Start,
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            },
            Limit::new(5)
        ).unwrap();

        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects{query}"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Projects>(response).await,
            Projects {
                projects: vec![
                    PROJECT_SUMMARY_A.clone(),
                    PROJECT_SUMMARY_B.clone()
                ],
                meta: Pagination {
                    prev_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::Before("project_a".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            Limit::new(5)
                        ).unwrap()
                    ),
                    next_page: Some(
                        SeekLink::new(
                            &Seek {
                                anchor: Anchor::After("project_b".into(), 0),
                                sort_by: SortBy::ProjectName,
                                dir: Direction::Ascending
                            },
                            Limit::new(5)
                        ).unwrap()
                    ),
                    total: 1234
                }
            }
        );
    }

    #[tokio::test]
    async fn get_project_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<ProjectData>(response).await,
            ProjectData {
                name: "eia".into(),
                description: "A module for Empires in Arms".into(),
                revision: 1,
                created_at: "2023-10-26T00:00:00,000000000+01:00".into(),
                modified_at: "2023-10-30T18:53:53,056386142+00:00".into(),
                tags: vec![],
                game: GameData {
                    title: "Empires in Arms".into(),
                    title_sort_key: "Empires in Arms".into(),
                    publisher: "Avalon Hill".into(),
                    year: "1983".into()
                },
                readme: "".into(),
                image: None,
                owners: vec!["alice".into(), "bob".into()],
// TODO: fill in more
                packages: vec![
                    PackageData {
                        name: "".into(),
                        description: "".into(),
                        releases: vec![
                            ReleaseData {
                                version: "".into(),
                                filename: "".into(),
                                url: "".into(),
                                size: 0,
                                checksum: "".into(),
                                published_at: "".into(),
                                published_by: "".into(),
                                requires: "".into(),
                                authors: vec![]
                            }
                        ]
                    }
                ]
            }
        );
    }

    #[tokio::test]
    async fn get_project_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn post_project_ok() {
        let proj_data = ProjectDataPost {
            description: "A module for Empires in Arms".into(),
            tags: vec![],
            game: GameData {
                title: "Empires in Arms".into(),
                title_sort_key: "Empires in Arms".into(),
                publisher: "Avalon Hill".into(),
                year: "1983".into()
            },
            readme: "".into(),
            image: None
        };

        let response = try_request(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("{API_V1}/projects/not_a_project"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&proj_data).unwrap()))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_empty(response).await);
    }

    #[tokio::test]
    async fn post_project_unauth() {
        let proj_data = ProjectDataPost {
            description: "A module for Empires in Arms".into(),
            tags: vec![],
            game: GameData {
                title: "Empires in Arms".into(),
                title_sort_key: "Empires in Arms".into(),
                publisher: "Avalon Hill".into(),
                year: "1983".into()
            },
            readme: "".into(),
            image: None
        };

        let response = try_request(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("{API_V1}/projects/not_a_project"))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&proj_data).unwrap()))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn post_project_wrong_json() {
        let response = try_request(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("{API_V1}/projects/not_a_project"))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from(r#"{ "garbage": "whatever" }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::JsonError)
        );
    }

    #[tokio::test]
    async fn post_project_wrong_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("{API_V1}/projects/not_a_project"))
                .header(CONTENT_TYPE, TEXT_PLAIN.as_ref())
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from("stuff"))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn post_project_no_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::POST)
                .uri(&format!("{API_V1}/projects/not_a_project"))
                .header(CONTENT_TYPE, TEXT_PLAIN.as_ref())
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from("stuff"))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn patch_project_ok() {
        let proj_data = ProjectDataPatch {
            description: Some("A module for Empires in Arms".into()),
            ..Default::default()
        };

        let response = try_request(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("{API_V1}/projects/a_project"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&proj_data).unwrap()))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_empty(response).await);
    }

    #[tokio::test]
    async fn patch_project_no_data() {
        let proj_data = ProjectDataPatch {
            ..Default::default()
        };

        let response = try_request(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("{API_V1}/projects/a_project"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&proj_data).unwrap()))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(),  StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::JsonError)
        );
    }

    #[tokio::test]
    async fn patch_project_unauth() {
        let proj_data = ProjectDataPatch {
            description: Some("A module for Empires in Arms".into()),
            ..Default::default()
        };

        let response = try_request(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("{API_V1}/projects/a_project"))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(serde_json::to_vec(&proj_data).unwrap()))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn patch_project_wrong_json() {
        let response = try_request(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("{API_V1}/projects/a_project"))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from(r#"{ "garbage": "whatever" }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::JsonError)
        );
    }

    #[tokio::test]
    async fn patch_project_wrong_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("{API_V1}/projects/a_project"))
                .header(CONTENT_TYPE, TEXT_PLAIN.as_ref())
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from("stuff"))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn patch_project_no_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::PATCH)
                .uri(&format!("{API_V1}/projects/a_project"))
                .header(CONTENT_TYPE, TEXT_PLAIN.as_ref())
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from("stuff"))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn get_project_revision_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/1"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<ProjectData>(response).await,
            ProjectData {
                name: "eia".into(),
                description: "A module for Empires in Arms".into(),
                revision: 1,
                created_at: "2023-10-26T00:00:00,000000000+01:00".into(),
                modified_at: "2023-10-30T18:53:53,056386142+00:00".into(),
                tags: vec![],
                game: GameData {
                    title: "Empires in Arms".into(),
                    title_sort_key: "Empires in Arms".into(),
                    publisher: "Avalon Hill".into(),
                    year: "1983".into()
                },
                readme: "".into(),
                image: None,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "".into(),
                        description: "".into(),
                        releases: vec![
                            ReleaseData {
                                version: "".into(),
                                filename: "".into(),
                                url: "".into(),
                                size: 0,
                                checksum: "".into(),
                                published_at: "".into(),
                                published_by: "".into(),
                                requires: "".into(),
                                authors: vec![]
                            }
                        ]
                    }
                ]
            }
        );
    }

    #[tokio::test]
    async fn get_project_revision_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project/1"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn get_project_revision_not_a_revision() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/2"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotARevision)
        );
    }

    #[tokio::test]
    async fn get_package_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/packages/a_package"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(LOCATION).unwrap(),
            "https://example.com/package"
        );
    }

    #[tokio::test]
    async fn get_package_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project/packages/a_package"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn get_package_not_a_package() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/packages/not_a_package"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAPackage)
        );
    }

    #[tokio::test]
    async fn get_release_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/packages/a_package/1.2.3"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(LOCATION).unwrap(),
            "https://example.com/package-1.2.3"
        );
    }

    #[tokio::test]
    async fn get_release_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project/packages/a_package/1.2.3"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn get_release_not_a_package() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/packages/not_a_package/1.2.3"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAPackage)
        );
    }

    #[tokio::test]
    async fn get_release_not_a_version() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/packages/a_package/bogus"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotFound)
        );
    }

    #[tokio::test]
    async fn get_owners_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Users>(response).await,
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into()
                ]
            }
        );
    }

    #[tokio::test]
    async fn get_owners_bad_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project/owners"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn put_owners_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_empty(response).await);
    }

    #[tokio::test]
    async fn put_owners_bad_project() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/not_a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn put_owners_unauth() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(0))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn put_owners_wrong_json() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "garbage": "whatever" }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::JsonError)
        );
    }

    #[tokio::test]
    async fn put_owners_wrong_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, TEXT_PLAIN.as_ref())
                .body(Body::from("stuff"))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn put_owners_no_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn delete_owners_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_empty(response).await);
    }

    #[tokio::test]
    async fn delete_owners_bad_project() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/not_a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn delete_owners_unauth() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(0))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn delete_owners_wrong_json() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                .body(Body::from(r#"{ "garbage": "whatever" }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::JsonError)
        );
    }

    #[tokio::test]
    async fn delete_owners_wrong_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .header(CONTENT_TYPE, TEXT_PLAIN.as_ref())
                .body(Body::from("stuff"))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn delete_owners_no_mime_type() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::BadMimeType)
        );
    }

    #[tokio::test]
    async fn get_players_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/players"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_as::<Users>(response).await,
            Users {
                users: vec![
                    "player 1".into(),
                    "player 2".into()
                ]
            }
        );
    }

    #[tokio::test]
    async fn get_players_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project/players"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn put_players_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/a_project/players"))
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_empty(response).await);
    }

    #[tokio::test]
    async fn put_players_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::PUT)
                .uri(&format!("{API_V1}/projects/not_a_project/owners"))
                .header(AUTHORIZATION, token(BOB_UID))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn delete_players_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/a_project/players"))
                .header(AUTHORIZATION, token(8))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_empty(response).await);
    }

    #[tokio::test]
    async fn delete_players_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::DELETE)
                .uri(&format!("{API_V1}/projects/not_a_project/players"))
                .header(AUTHORIZATION, token(8))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }

    #[tokio::test]
    async fn get_image_ok() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/a_project/images/img.png"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(LOCATION).unwrap(),
            "https://example.com/img.png"
        );
    }

    #[tokio::test]
    async fn get_image_not_a_project() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/projects/not_a_project/images/img.png"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotAProject)
        );
    }


    #[tokio::test]
    async fn bad_path() {
        let response = try_request(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("{API_V1}/bogus/whatever"))
                .body(Body::empty())
                .unwrap()
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_as::<HttpError>(response).await,
            HttpError::from(AppError::NotFound)
        );
    }
}
