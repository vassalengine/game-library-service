use jsonwebtoken::{DecodingKey, Validation};
use serde::Deserialize;

use crate::{errors::AppError};

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized
    }
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64
}

#[derive(Clone)]
pub struct Key(pub DecodingKey);

pub fn verify(token: &str, key: &Key) -> Result<Claims, AppError> {
    Ok(
        jsonwebtoken::decode::<Claims>(
            token,
            &key.0,
            &Validation::default()
        )?.claims
    )
}
