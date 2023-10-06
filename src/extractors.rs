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
    errors::AppError,
    jwt::{self, Claims, Key},
    model::Owner,
    queries::{Database, user_is_owner}
};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    Key: FromRef<S>
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // get the bearer token from the Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader::<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // verify the token
        let key = Key::from_ref(state);
        let claims = jwt::verify(bearer.token(), &key)?;

        Ok(claims)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Owner
where
    S: Send + Sync,
    Key: FromRef<S>,
    Database: FromRef<S>
{
    type Rejection = AppError;

     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(proj_id) = Path::<u32>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        let claims = Claims::from_request_parts(parts, state).await?;

// TODO: can we wrap the pool in our own struct?
        let State(db) = State::<Database>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        match user_is_owner(&claims.sub, proj_id, &db).await? {
            true => Ok(Owner(claims.sub)),
            false =>  Err(AppError::Unauthorized)
        }
    }
}
