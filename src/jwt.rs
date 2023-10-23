use jsonwebtoken::{Header, Validation, get_current_timestamp};
use serde::{Deserialize, Serialize};

use crate::{errors::AppError};

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64
}

#[derive(Clone)]
pub struct DecodingKey(jsonwebtoken::DecodingKey);

impl DecodingKey {
    pub fn from_secret(secret: &[u8]) -> Self {
        DecodingKey(jsonwebtoken::DecodingKey::from_secret(secret))
    }
}

pub fn verify(token: &str, key: &DecodingKey) -> Result<Claims, AppError> {
    Ok(
        jsonwebtoken::decode::<Claims>(
            token,
            &key.0,
            &Validation::default()
        )?.claims
    )
}

#[derive(Clone)]
pub struct EncodingKey(jsonwebtoken::EncodingKey);

impl EncodingKey {
    pub fn from_secret(secret: &[u8]) -> Self {
        EncodingKey(jsonwebtoken::EncodingKey::from_secret(secret))
    }
}


pub fn issue(key: &EncodingKey, username: &str, expiry: u64) -> Result<String, AppError> {
    let claims = Claims {
        sub: username.into(),
        exp: expiry,
        iat: get_current_timestamp()
    };

    Ok(jsonwebtoken::encode(&Header::default(), &claims, &key.0)?)
}
