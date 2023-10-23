use axum::{
    async_trait, RequestPartsExt, TypedHeader,
    extract::{FromRequestParts, FromRef, Path, State},
    headers::{
        Authorization,
        authorization::Bearer
    },
    http::request::Parts,
};

use crate::{
    core::CoreArc,
    errors::AppError,
    jwt::{self, Claims, DecodingKey},
    model::{Owner, User}
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

#[async_trait]
impl<S> FromRequestParts<S> for Owner
where
    S: Send + Sync,
    DecodingKey: FromRef<S>,
    CoreArc: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // check that the requester is authorized
        let claims = Claims::from_request_parts(parts, state).await?;

        // should never fail
        let Path(proj_id) = Path::<u32>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // should never fail
        let State(core) = State::<CoreArc>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        // check that that requester owns the project
        let requester = User(claims.sub.clone());

        match core.user_is_owner(&requester, proj_id).await? {
            true => Ok(Owner(claims.sub)),
            false =>  Err(AppError::Unauthorized)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use axum::{
        Router,
        body::{boxed, Empty},
        http::{
            Method, Request, StatusCode,
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

    fn make_auth(key: &[u8], claims: &Claims) -> String {
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
            .header(AUTHORIZATION, make_auth(KEY, &exp))
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
            .header(AUTHORIZATION, make_auth(KEY, &exp))
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
            .header(AUTHORIZATION, make_auth(b"wrong key", &exp))
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
            .header(AUTHORIZATION, make_auth(KEY, &exp))
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
            .header(AUTHORIZATION, make_auth(KEY, &exp))
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
            .header(AUTHORIZATION, make_auth(b"wrong key", &exp))
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

    #[derive(Clone)]
    struct TestCore {}

    #[axum::async_trait]
    impl Core for TestCore {
        async fn user_is_owner(
            &self,
            user: &User,
            proj_id: u32
        ) -> Result<bool, AppError>
        {
            Ok(proj_id == 42 && user == &User("bob".into()))
        }

        async fn get_owners(
            &self,
            _proj_id: u32
        ) -> Result<Users, AppError>
        {
            Ok(
                Users {
                    users: vec!(
                        User("bob".into())
                    )
                }
            )
        }
    }

    async fn owner_ok(
        requester: Owner,
        _: Path<u32>,
        State(_): State<AppState>
    )
    {
        assert_eq!(requester, Owner("bob".into()));
    }

    async fn owner_fail(
        _: Owner,
        _: Path<u32>,
        State(_): State<AppState>
    )
    {
        unreachable!();
    }

    fn make_state() -> AppState {
        AppState {
            key: DecodingKey::from_secret(KEY),
            core: Arc::new(TestCore {}) as CoreArc
        }
    }

    // We have to test Owner::from_request_parts via a Router because
    // Path uses a private extension to get parameters from the request

    #[tokio::test]
    async fn owners_from_request_parts_ok() {
        let exp = bob_ok();

        let app = Router::new()
            .route("/:proj_id", get(owner_ok))
            .with_state(make_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/42")
                    .header(AUTHORIZATION, make_auth(KEY, &exp))
                    .body(boxed(Empty::new()))
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
            .route("/:proj_id", get(owner_fail))
            .with_state(make_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/42")
                    .header(AUTHORIZATION, make_auth(KEY, &exp))
                    .body(boxed(Empty::new()))
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
            .route("/:proj_id", get(owner_fail))
            .with_state(make_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/42")
                    .header(AUTHORIZATION, make_auth(KEY, &exp))
                    .body(boxed(Empty::new()))
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
            .route("/:proj_id", get(owner_fail))
            .with_state(make_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/42")
                    .header(AUTHORIZATION, make_auth(b"wrong key", &exp))
                    .body(boxed(Empty::new()))
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn owners_from_request_parts_no_token() {
        let app = Router::new()
            .route("/:proj_id", get(owner_fail))
            .with_state(make_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/42")
                    .header(AUTHORIZATION, "")
                    .body(boxed(Empty::new()))
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn owners_from_request_parts_no_auth_header() {
        let app = Router::new()
            .route("/:proj_id", get(owner_fail))
            .with_state(make_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/42")
                    .body(boxed(Empty::new()))
                    .unwrap()
             )
             .await
             .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
