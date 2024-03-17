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
    model::{Owned, Owner, Package, Project, User}
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
            .or(Err(AppError::Unauthorized))?;

        // verify the token
        let key = DecodingKey::from_ref(state);
        let claims = jwt::verify(bearer.token(), &key)
            .or(Err(AppError::Unauthorized))?;

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
        // extract the user id
        Ok(User(claims.sub))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Project
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

        // look up the project id
        Ok(core.get_project_id(&proj).await?)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Package
where
    S: Send + Sync,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that that project exists
        let proj = Project::from_request_parts(parts, state).await?;

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
        Ok(core.get_package_id(proj, &pkg).await?)
    }
}

pub struct ProjectAndPackage(pub (Project, Package));

#[async_trait]
impl<S> FromRequestParts<S> for ProjectAndPackage
where
    S: Send + Sync,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that that project exists
        let proj = Project::from_request_parts(parts, state).await?;

        let pkg = Package::from_request_parts(parts, state).await?;

        Ok(ProjectAndPackage((proj, pkg)))
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
        let proj = Project::from_request_parts(parts, state).await?;

        // should never fail
        let State(core) = State::<CoreArc>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // check that that requester owns the project
        let requester = User(claims.sub);

        match core.user_is_owner(requester, proj).await? {
            true => Ok(Owned(Owner(claims.sub), proj)),
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
        core::{Core, CoreError},
        jwt::EncodingKey,
        model::Users
    };

    const KEY: &[u8] = b"@wlD+3L)EHdv28u)OFWx@83_*TxhVf9IdUncaAz6ICbM~)j+dH=sR2^LXp(tW31z";

    fn bob_ok() -> Claims {
        Claims {
            sub: 1,
            exp: 899999999999,
            iat: 0
        }
    }

    fn bob_expired() -> Claims {
        Claims {
            sub: 1,
            exp: 0,
            iat: 0
        }
    }

    fn token(key: &[u8], claims: &Claims) -> String {
        let ekey = EncodingKey::from_secret(key);
        let token = jwt::issue(&ekey, claims.sub, claims.exp).unwrap();
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

    // We have to test Project::from_request_parts via a Router because
    // Path uses a private extension to get parameters from the request

    #[derive(Clone)]
    struct ProjectTestCore {}

    #[axum::async_trait]
    impl Core for ProjectTestCore {
        async fn get_project_id(
            &self,
            proj: &str
        ) -> Result<Project, CoreError>
        {
            match proj {
                "a_project" => Ok(Project(42)),
                _ => Err(CoreError::NotAProject)
            }
        }
    }

    async fn project_ok(
        proj: Project,
        State(_): State<AppState>
    )
    {
        assert_eq!(proj, Project(42));
    }

    async fn project_fail(
        _proj: Project,
        State(_): State<AppState>
    )
    {
        unreachable!();
    }

    #[tokio::test]
    async fn project_id_from_request_parts_ok() {
        let app = Router::new()
            .route("/:proj", get(project_ok))
            .with_state(make_state(ProjectTestCore {}));

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
            .with_state(make_state(ProjectTestCore {}));

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
            proj: &str
        ) -> Result<Project, CoreError>
        {
            match proj {
                "a_project" => Ok(Project(42)),
                _ => Err(CoreError::NotAProject)
            }
        }

        async fn user_is_owner(
            &self,
            user: User,
            proj: Project
        ) -> Result<bool, CoreError>
        {
            Ok(user == User(1) && proj == Project(42))
        }

        async fn get_owners(
            &self,
            _proj: Project
        ) -> Result<Users, CoreError>
        {
            Ok(Users { users: vec!["bob".into()] })
        }
    }

    async fn owned_ok(
        owned: Owned,
        State(_): State<AppState>
    )
    {
        assert_eq!(owned, Owned(Owner(1), Project(42)));
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
            sub: 2,
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
