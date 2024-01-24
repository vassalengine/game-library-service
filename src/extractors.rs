use axum::{
    async_trait, RequestPartsExt,
    extract::{
        FromRequest, FromRequestParts, FromRef, Path, Request, State,
        rejection::{JsonRejection, QueryRejection}
    },
    http::request::Parts
};
use axum_extra::{
    TypedHeader,
    headers::{
        Authorization,
        authorization::Bearer
    }
};

use crate::{
    core::CoreArc,
    errors::AppError,
    jwt::{self, Claims, DecodingKey},
    model::{Owned, Owner, PackageID, Project, ProjectID, User}
};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    DecodingKey: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // get the bearer token from the Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader::<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // verify the token
        let key = DecodingKey::from_ref(state);
        let claims = jwt::verify(bearer.token(), &key)?;

        Ok(claims)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    DecodingKey: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that the requester is authorized
        let claims = Claims::from_request_parts(parts, state).await?;
        // extract the username
        Ok(User(claims.sub))
    }
}

/*
#[async_trait]
impl<S> FromRequestParts<S> for UserID
where
    S: Send + Sync,
    DecodingKey: FromRef<S>,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // extract the user
        let user = User::from_request_parts(parts, state).await?;

        // should never fail
        let State(core) = State::<CoreArc>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // lookup the user id
        Ok(core.get_user_id(&user).await?)
    }
}
*/

#[async_trait]
impl<S> FromRequestParts<S> for ProjectID
where
    S: Send + Sync,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // extract however many path elements there are; should never fail
        let Path(params) =
            Path::<Vec<(String, String)>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::InternalError)?;

        // extract the first path element, which is the project name
        let (_, proj) = params
            .into_iter()
            .next()
            .ok_or(AppError::InternalError)?;

        // should never fail
        let State(core) = State::<CoreArc>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // look up the user id
        Ok(core.get_project_id(&Project(proj)).await?)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for PackageID
where
    S: Send + Sync,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that that project exists
        let proj_id = ProjectID::from_request_parts(parts, state).await?;

        // extract however many path elements there are; should never fail
        let Path(params) =
            Path::<Vec<(String, String)>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::InternalError)?;

        // extract the second path element, which is the package name
        let (_, pkg) = params
            .into_iter()
            .nth(1)
            .ok_or(AppError::InternalError)?;

        // should never fail
        let State(core) = State::<CoreArc>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // look up the package id
        Ok(core.get_package_id(proj_id.0, &pkg).await?)
    }
}

pub struct ProjectIDAndPackageID(pub (ProjectID, PackageID));

#[async_trait]
impl<S> FromRequestParts<S> for ProjectIDAndPackageID
where
    S: Send + Sync,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that that project exists
        let proj_id = ProjectID::from_request_parts(parts, state).await?;

        let pkg_id = PackageID::from_request_parts(parts, state).await?;

        Ok(ProjectIDAndPackageID((proj_id, pkg_id)))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Owned
where
    S: Send + Sync,
    DecodingKey: FromRef<S>,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that the requester is authorized
        let claims = Claims::from_request_parts(parts, state).await?;

        // check that that project exists
        let proj_id = ProjectID::from_request_parts(parts, state).await?;

        // should never fail
        let State(core) = State::<CoreArc>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // check that that requester owns the project
        let requester = User(claims.sub.clone());

        match core.user_is_owner(&requester, proj_id.0).await? {
            true => Ok(Owned(Owner(claims.sub), proj_id)),
            false =>  Err(AppError::Unauthorized)
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        match err {
            JsonRejection::MissingJsonContentType(_) => AppError::BadMimeType,
            _ => AppError::JsonError
        }
    }
}

impl From<QueryRejection> for AppError {
    fn from(_: QueryRejection) -> Self {
       AppError::MalformedQuery
    }
}

pub struct Wrapper<E>(pub E);

#[async_trait]
impl<S, T> FromRequestParts<S> for Wrapper<T>
where
    S: Send + Sync,
    T: FromRequestParts<S>,
    AppError: From<<T as FromRequestParts<S>>::Rejection>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Wrapper(T::from_request_parts(parts, state).await?))
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for Wrapper<T>
where
    S: Send + Sync,
    T: FromRequest<S>,
    AppError: From<<T as FromRequest<S>>::Rejection>
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Wrapper(T::from_request(req, state).await?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use axum::{
        Router,
        body::Body,
        http::{
            Method, StatusCode,
            header::AUTHORIZATION
        },
        routing::get
    };
    use std::sync::Arc;
    use tower::ServiceExt; // for oneshot

    use crate::{
        app::AppState,
        core::Core,
        jwt::EncodingKey,
        model::Users
    };

    const KEY: &[u8] = b"@wlD+3L)EHdv28u)OFWx@83_*TxhVf9IdUncaAz6ICbM~)j+dH=sR2^LXp(tW31z";

    fn bob_ok() -> Claims {
        Claims {
            sub: "bob".into(),
            exp: 899999999999,
            iat: 0
        }
    }

    fn bob_expired() -> Claims {
        Claims {
            sub: "bob".into(),
            exp: 0,
            iat: 0
        }
    }

    fn token(key: &[u8], claims: &Claims) -> String {
        let ekey = EncodingKey::from_secret(key);
        let token = jwt::issue(&ekey, &claims.sub, claims.exp).unwrap();
        format!("Bearer {token}")
    }

    #[tokio::test]
    async fn claims_from_request_parts_ok() {
        let exp = bob_ok();
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, token(KEY, &exp))
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = Claims::from_request_parts(&mut parts, &dkey).await.unwrap();
        assert_eq!(act.sub, exp.sub);
        assert_eq!(act.exp, exp.exp);
        // TODO: check iat
    }

    #[tokio::test]
    async fn claims_from_request_parts_expired() {
        let exp = bob_expired();
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, token(KEY, &exp))
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = Claims::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn claims_from_request_parts_wrong_key() {
        let exp = bob_ok();
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, token(b"wrong key", &exp))
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = Claims::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn claims_from_request_parts_no_token() {
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, "")
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = Claims::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn claims_from_request_parts_no_auth_header() {
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = Claims::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn user_from_request_parts_ok() {
        let exp = bob_ok();
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, token(KEY, &exp))
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = User::from_request_parts(&mut parts, &dkey).await.unwrap();
        assert_eq!(act, User(exp.sub));
    }

    #[tokio::test]
    async fn user_from_request_parts_expired() {
        let exp = bob_expired();
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, token(KEY, &exp))
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = User::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn user_from_request_parts_wrong_key() {
        let exp = bob_ok();
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, token(b"wrong key", &exp))
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = User::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn user_from_request_parts_no_token() {
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header(AUTHORIZATION, "")
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = User::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn user_from_request_parts_no_auth_header() {
        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(())
            .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = User::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    fn make_state(core: impl Core + Send + Sync + 'static) -> AppState {
        AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(core) as CoreArc
        }
    }

    // We have to test ProjectID::from_request_parts via a Router because
    // Path uses a private extension to get parameters from the request

    #[derive(Clone)]
    struct ProjectIDTestCore {}

    #[axum::async_trait]
    impl Core for ProjectIDTestCore {
        async fn get_project_id(
            &self,
            proj: &Project
        ) -> Result<ProjectID, AppError>
        {
            match proj.0.as_str() {
                "a_project" => Ok(ProjectID(42)),
                _ => Err(AppError::NotAProject)
            }
        }
    }

    async fn project_ok(
        proj_id: ProjectID,
        State(_): State<AppState>
    )
    {
        assert_eq!(proj_id, ProjectID(42));
    }

    async fn project_fail(
        _proj_id: ProjectID,
        State(_): State<AppState>
    )
    {
        unreachable!();
    }

    #[tokio::test]
    async fn project_id_from_request_parts_ok() {
        let app = Router::new()
            .route("/:proj", get(project_ok))
            .with_state(make_state(ProjectIDTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn project_id_from_request_parts_not_a_project() {
        let app = Router::new()
            .route("/:proj", get(project_fail))
            .with_state(make_state(ProjectIDTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/not_a_project")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // We have to test Owner::from_request_parts via a Router because
    // Path uses a private extension to get parameters from the request

    #[derive(Clone)]
    struct OwnersTestCore {}

    #[axum::async_trait]
    impl Core for OwnersTestCore {
        async fn get_project_id(
            &self,
            proj: &Project
        ) -> Result<ProjectID, AppError>
        {
            match proj.0.as_str() {
                "a_project" => Ok(ProjectID(42)),
                _ => Err(AppError::NotAProject)
            }
        }

        async fn user_is_owner(
            &self,
            user: &User,
            proj_id: i64
        ) -> Result<bool, AppError>
        {
            Ok(proj_id == 42 && user == &User("bob".into()))
        }

        async fn get_owners(
            &self,
            _proj_id: i64
        ) -> Result<Users, AppError>
        {
            Ok(
                Users {
                    users: vec![
                        User("bob".into())
                    ]
                }
            )
        }
    }

    async fn owned_ok(
        owned: Owned,
        State(_): State<AppState>
    )
    {
        assert_eq!(owned, Owned(Owner("bob".into()), ProjectID(42)));
    }

    async fn owned_fail(
        _: Owned,
        State(_): State<AppState>
    )
    {
        unreachable!();
    }

    #[tokio::test]
    async fn owners_from_request_parts_ok() {
        let exp = bob_ok();

        let app = Router::new()
            .route("/:proj", get(owned_ok))
            .with_state(make_state(OwnersTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .header(AUTHORIZATION, token(KEY, &exp))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn owners_from_request_parts_not_owner() {
        let exp = Claims {
            sub: "alice".into(),
            exp: 899999999999,
            iat: 0
        };

        let app = Router::new()
            .route("/:proj", get(owned_fail))
            .with_state(make_state(OwnersTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .header(AUTHORIZATION, token(KEY, &exp))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn owners_from_request_parts_expired() {
        let exp = bob_expired();

        let app = Router::new()
            .route("/:proj", get(owned_fail))
            .with_state(make_state(OwnersTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .header(AUTHORIZATION, token(KEY, &exp))
                    .body(Body::empty())
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn owners_from_request_parts_wrong_key() {
        let exp = bob_ok();

        let app = Router::new()
            .route("/:proj", get(owned_fail))
            .with_state(make_state(OwnersTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .header(AUTHORIZATION, token(b"wrong key", &exp))
                    .body(Body::empty())
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn owners_from_request_parts_no_token() {
        let app = Router::new()
            .route("/:proj", get(owned_fail))
            .with_state(make_state(OwnersTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .header(AUTHORIZATION, "")
                    .body(Body::empty())
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn owners_from_request_parts_no_auth_header() {
        let app = Router::new()
            .route("/:proj", get(owned_fail))
            .with_state(make_state(OwnersTestCore {}));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/a_project")
                    .body(Body::empty())
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
