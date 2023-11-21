use base64::{Engine as _};
use serde::{Deserialize, Serialize};
use std::str;
use std::num::NonZeroU8;

use crate::errors::AppError;

// FIXME: private fields various places

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "String")]
#[repr(transparent)]
pub struct Limit(NonZeroU8);

impl Limit {
    pub const fn new(limit: u8) -> Option<Limit> {
        match limit {
            limit if limit > 100 => None,
            limit => match NonZeroU8::new(limit) {
                Some(n) => Some(Limit(n)),
                None => None
            }
        }
    }

    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

impl Default for Limit {
    fn default() -> Self {
        Limit::new(10).expect("0 < 10 <= 100")
    }
}

impl TryFrom<String> for Limit {
    type Error = AppError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.parse::<u8>() {
            Ok(n) => Limit::new(n).ok_or(AppError::LimitOutOfRange),
            Err(_) => Err(AppError::MalformedQuery)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub enum Seek {
    Start,
    Before(String),
    After(String),
    End
}

impl Default for Seek {
    fn default() -> Self {
        Seek::Start
    }
}

impl From<Seek> for String {
    fn from(value: Seek) -> Self {
        let s = match value {
            Seek::Start => "s:".to_string(),
            Seek::Before(s) => format!("b:{s}"),
            Seek::After(s) => format!("a:{s}"),
            Seek::End => "e:".to_string()
        };

        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(s)
    }
}

// TODO: can we do this zero-copy?
impl TryFrom<String> for Seek {
    type Error = AppError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let buf = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| AppError::MalformedQuery)?;

        let d = str::from_utf8(&buf)
            .map_err(|_| AppError::MalformedQuery)?;

        match d.split_once(':') {
            Some(("s", "")) => Ok(Seek::Start),
            Some(("b", n)) => Ok(Seek::Before(n.into())),
            Some(("a", n)) => Ok(Seek::After(n.into())),
            Some(("e", "")) => Ok(Seek::End),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub seek: Option<Seek>,
    pub limit: Option<Limit>
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size_of_limit() {
        assert_eq!(
            std::mem::size_of::<Limit>(),
            std::mem::size_of::<u8>()
        );
    }
}
