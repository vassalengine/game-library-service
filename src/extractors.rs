use axum::{
    async_trait, RequestPartsExt, TypedHeader,
    extract::{FromRequestParts, FromRef, Path, State},
    headers::{
        Authorization,
        authorization::Bearer
    },
    http::request::Parts,
};
use std::sync::Arc;

use crate::{
    datastore::DataStore,
    errors::AppError,
    jwt::{self, Claims, DecodingKey},
    model::{Owner, User}
};

type DS = Arc<dyn DataStore + Send + Sync>;

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
impl<S> FromRequestParts<S> for Owner
where
    S: Send + Sync,
    DecodingKey: FromRef<S>,
    DS: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(proj_id) = Path::<u32>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        let claims = Claims::from_request_parts(parts, state).await?;

        let State(db) = State::<DS>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        let requester = User(claims.sub.clone());

        match db.user_is_owner(&requester, proj_id).await? {
            true => Ok(Owner(claims.sub)),
            false =>  Err(AppError::Unauthorized)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
   
    use axum::http::{
        Method, Request,
        header::AUTHORIZATION,
    };

    use crate::{
      jwt::{self, EncodingKey}
    };

    const KEY: &[u8] = b"@wlD+3L)EHdv28u)OFWx@83_*TxhVf9IdUncaAz6ICbM~)j+dH=sR2^LXp(tW31z";

    #[tokio::test]
    async fn claims_from_request_parts_ok() {
        let exp = Claims {
            sub: "bob".into(),
            exp: 899999999999,
            iat: 0
        };

        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, &exp.sub, exp.exp).unwrap();
        let auth = format!("Bearer {token}");

        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(AUTHORIZATION, auth)
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
        let exp = Claims {
            sub: "bob".into(),
            exp: 0,
            iat: 0
        };

        let ekey = EncodingKey::from_secret(KEY);
        let token = jwt::issue(&ekey, &exp.sub, exp.exp).unwrap();
        let auth = format!("Bearer {token}");

        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(AUTHORIZATION, auth)
                    .body(())
                    .unwrap();

        let mut parts;
        (parts, _) = request.into_parts();

        let act = Claims::from_request_parts(&mut parts, &dkey).await;
        assert!(act.is_err());
    }

    #[tokio::test]
    async fn claims_from_request_parts_wrong_key() {
        let exp = Claims {
            sub: "bob".into(),
            exp: 0,
            iat: 0
        };

        let ekey = EncodingKey::from_secret(b"other key");
        let token = jwt::issue(&ekey, &exp.sub, exp.exp).unwrap();
        let auth = format!("Bearer {token}");

        let dkey = DecodingKey::from_secret(KEY);

        let request = Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(AUTHORIZATION, auth)
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
    async fn claims_from_request_parts_no_header() {
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

}
