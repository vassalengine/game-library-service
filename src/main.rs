#![feature(async_fn_in_trait)]

use axum::{
    Router, Server,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get
};
//use base64::{Engine, engine::general_purpose};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use std::{
    net::SocketAddr,
    sync::Arc
};

mod app;
mod config;
mod core;
mod datastore;
mod db;
mod errors;
mod extractors;
mod handlers;
mod jwt;
mod model;
mod prod_core;

use crate::{
    app::AppState,
    config::Config,
    core::Core,
    prod_core::ProdCore,
    datastore::DataStoreError,
    errors::AppError,
    jwt::DecodingKey,
    handlers::{
        root_get,
        projects_get,
        project_get,
        project_update,
        project_revision_get,
        owners_get,
        owners_add,
        owners_remove,
        players_get,
        players_add,
        players_remove,
        package_get,
        package_version_get,
        package_version_put,
        readme_get,
        readme_revision_get,
        image_get,
        image_put
    }
};

impl From<DataStoreError> for AppError {
    fn from(e: DataStoreError) -> Self {
        match e {
            DataStoreError::Problem(e) => AppError::DatabaseError(e.to_string())
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
/*
            AppError::BadPagination => {
                (StatusCode::BAD_REQUEST, "Bad pagination".into())
            },
*/
            AppError::DatabaseError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e)
            },
            AppError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into())
            },
            AppError::NotImplemented => {
                (StatusCode::NOT_IMPLEMENTED, "Not implemented".into())
            },
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".into())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

fn routes(api: &str) -> Router<AppState> {
    Router::new()
        .route(
            &format!("{api}/"),
            get(root_get)
        )
        .route(
            &format!("{api}/projects"),
            get(projects_get)
        )
        .route(&format!(
            "{api}/projects/:proj_id"),
            get(project_get)
            .put(project_update)
        )
        .route(
            &format!("{api}/projects/:proj_id/:revision"),
            get(project_revision_get)
        )
        .route(
            &format!("{api}/projects/:proj_id/owners"),
            get(owners_get)
            .put(owners_add)
            .delete(owners_remove)
        )
        .route(
            &format!("{api}/projects/:proj_id/players"),
            get(players_get)
            .put(players_add)
            .delete(players_remove)
        )
        .route(
            &format!("{api}/projects/:proj_id/packages/:pkg_name"),
            get(package_get)
        )
        .route(
            &format!("{api}/projects/:proj_id/packages/:pkg_name/:version"),
            get(package_version_get)
            .put(package_version_put)
        )
        .route(
            &format!("{api}/projects/:proj_id/readme"),
            get(readme_get)
        )
        .route(
            &format!("{api}/projects/:proj_id/readme/:revision"),
            get(readme_revision_get)
        )
        .route(
            &format!("{api}/projects/:proj_id/images/:img_name"),
            get(image_get)
            .put(image_put)
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

    let core = ProdCore { db: db_pool };

    let state = AppState {
        key: DecodingKey::from_secret(&config.jwt_key),
        core: Arc::new(core) as Arc<dyn Core + Send + Sync>
    };

    let api = &config.api_base_path;

    let app: Router = routes(api).with_state(state);

    let addr = SocketAddr::from((config.listen_ip, config.listen_port));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    use axum::{
        body::Body,
        http::{
            Method, Request,
            header::{AUTHORIZATION, CONTENT_TYPE}
        }
    };
    use mime::APPLICATION_JSON;
    use serde::Deserialize;
    use tower::ServiceExt; // for oneshot

    use crate::{
      jwt::{self, EncodingKey},
      model::{User, Users}
    };

    const API_V1: &str = "/api/v1";
    const KEY: &[u8] = b"@wlD+3L)EHdv28u)OFWx@83_*TxhVf9IdUncaAz6ICbM~)j+dH=sR2^LXp(tW31z";

    #[derive(Debug, Deserialize, PartialEq)]
    struct HttpError {
        error: String
    }

    #[derive(Clone)]
    struct UnimplementedCore {}

    #[axum::async_trait]
    impl Core for UnimplementedCore {
        async fn user_is_owner(
            &self,
            _user: &User,
            _proj_id: u32
        ) -> Result<bool, AppError>
        {
            unimplemented!()
        }

        async fn add_owners(
            &self,
            _owners: &Users,
            _proj_id: u32
        ) -> Result<(), AppError>
        {
            unimplemented!()
        }

        async fn remove_owners(
            &self,
            _owners: &Users,
            _proj_id: u32
        ) -> Result<(), AppError>
        {
            unimplemented!()
        }

        async fn get_owners(
            &self,
            _proj_id: u32
        ) -> Result<Users, AppError>
        {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn root_ok() {
        let state = AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(UnimplementedCore {}) as Arc<dyn Core + Send + Sync>

        };

        let app: Router = routes(API_V1).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&format!("{API_V1}/"))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"hello world");
    }

    #[derive(Clone)]
    struct TestCore { }

    #[axum::async_trait]
    impl Core for TestCore {
        async fn user_is_owner(
            &self,
            user: &User,
            _proj_id: u32
        ) -> Result<bool, AppError>
        {
            Ok(user == &User("bob".into()) || user == &User("alice".into()))
        }

        async fn add_owners(
            &self,
            _owners: &Users,
            _proj_id: u32
        ) -> Result<(), AppError>
        {
            Ok(())
        }

        async fn remove_owners(
            &self,
            _owners: &Users,
            _proj_id: u32
        ) -> Result<(), AppError>
        {
            Ok(())
        }

        async fn get_owners(
            &self,
            _proj_id: u32
        ) -> Result<Users, AppError>
        {
            Ok(
                Users {
                    users: vec!(
                        User("alice".into()),
                        User("bob".into())
                    )
                }
            )
        }
    }

    #[tokio::test]
    async fn get_owners_ok() {
        let state = AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as Arc<dyn Core + Send + Sync>
        };

        let app = routes(API_V1).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&format!("{API_V1}/projects/1/owners"))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Users = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            Users {
                users: vec!(
                    User("alice".into()),
                    User("bob".into())
                )
            }
        );
    }

    #[tokio::test]
    async fn put_owners_ok() {
        let state = AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as Arc<dyn Core + Send + Sync>
        };

        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, "bob", 899999999999).unwrap();
        let auth = format!("Bearer {token}");

        let app = routes(API_V1).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(&format!("{API_V1}/projects/1/owners"))
                    .header(AUTHORIZATION, auth)
                    .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                    .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body.len(), 0);
    }

    #[tokio::test]
    async fn put_owners_unauth() {
        let state = AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as Arc<dyn Core + Send + Sync>
        };

        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, "rando", 899999999999).unwrap();
        let auth = format!("Bearer {token}");

        let app = routes(API_V1).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(&format!("{API_V1}/projects/1/owners"))
                    .header(AUTHORIZATION, auth)
                    .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                    .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: HttpError = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            HttpError { error: "Unauthorized".into() }
        );
    }

    #[tokio::test]
    async fn delete_owners_ok() {
        let state = AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as Arc<dyn Core + Send + Sync>
        };

        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, "bob", 899999999999).unwrap();
        let auth = format!("Bearer {token}");

        let app = routes(API_V1).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(&format!("{API_V1}/projects/1/owners"))
                    .header(AUTHORIZATION, auth)
                    .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                    .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(body.len(), 0);
    }

    #[tokio::test]
    async fn delete_owners_unauth() {
        let state = AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as Arc<dyn Core + Send + Sync>
        };

        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, "rando", 899999999999).unwrap();
        let auth = format!("Bearer {token}");

        let app = routes(API_V1).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(&format!("{API_V1}/projects/1/owners"))
                    .header(AUTHORIZATION, auth)
                    .header(CONTENT_TYPE, APPLICATION_JSON.as_ref())
                    .body(Body::from(r#"{ "users": ["alice", "bob"] }"#))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: HttpError = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            body,
            HttpError { error: "Unauthorized".into() }
        );
    }

    #[tokio::test]
    async fn get_players_ok() {
    }

    #[tokio::test]
    async fn put_players_ok() {
    }

    #[tokio::test]
    async fn delete_players_ok() {
    }
}
