use jsonwebtoken::{Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("{0}")]
pub struct Error(#[from] jsonwebtoken::errors::Error);

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Claims {
    pub sub: i64,
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

pub fn verify(token: &str, key: &DecodingKey) -> Result<Claims, Error> {
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


pub fn issue(
    key: &EncodingKey,
    uid: i64,
    now: u64,
    expiry: u64
) -> Result<String, Error>
{
    let claims = Claims {
        sub: uid,
        exp: expiry,
        iat: now
    };

    Ok(jsonwebtoken::encode(&Header::default(), &claims, &key.0)?)
}
