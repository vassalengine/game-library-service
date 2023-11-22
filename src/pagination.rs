use base64::{Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    str,
    num::NonZeroU8
};

use crate::errors::AppError;

// TODO: private fields various places

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
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

impl TryFrom<&str> for Limit {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.parse::<u8>() {
            Ok(n) => Limit::new(n).ok_or(AppError::LimitOutOfRange),
            Err(_) => Err(AppError::MalformedQuery)
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub enum Seek {
    #[default]
    Start,
    Before(String),
    After(String),
    End
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

impl TryFrom<&str> for Seek {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
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

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SeekLink(pub String);

impl From<Seek> for SeekLink {
    fn from(seek: Seek) -> Self {
        SeekLink(format!("/?seek={}", String::from(seek)))
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Pagination {
    pub prev_page: Option<SeekLink>,
    pub next_page: Option<SeekLink>,
    pub total: i32
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

    #[test]
    fn string_to_limit_zero_err() {
        assert!(Limit::try_from("0").is_err());
    }

    #[test]
    fn string_to_limit_one_ok() {
        assert_eq!(
            Limit::try_from("1").unwrap(),
            Limit::new(1).unwrap()
        );
    }

    #[test]
    fn string_to_limit_one_hundred_ok() {
        assert_eq!(
            Limit::try_from("100").unwrap(),
            Limit::new(100).unwrap()
        );
    }

    #[test]
    fn string_to_limit_one_hundred_one_err() {
        assert!(Limit::try_from("101").is_err());
    }

    #[test]
    fn seek_to_string_start() {
        assert_eq!(
            String::from(Seek::Start),
            "czo".to_string()
        );
    }

    #[test]
    fn seek_to_string_end() {
        assert_eq!(
            String::from(Seek::End),
            "ZTo".to_string()
        );
    }

    #[test]
    fn seek_to_string_before() {
        assert_eq!(
            String::from(Seek::Before("abc".into())),
            "YjphYmM".to_string()
        );
    }

    #[test]
    fn seek_to_string_after() {
        assert_eq!(
            String::from(Seek::After("abc".into())),
            "YTphYmM".to_string()
        );
    }

    #[test]
    fn string_to_seek_start() {
        assert_eq!(
            Seek::try_from("czo").unwrap(),
            Seek::Start
        );
    }

    #[test]
    fn string_to_seek_end() {
        assert_eq!(
            Seek::try_from("ZTo").unwrap(),
            Seek::End
        );
    }

    #[test]
    fn string_to_seek_before() {
        assert_eq!(
            Seek::try_from("YjphYmM").unwrap(),
            Seek::Before("abc".into())
        );
    }

    #[test]
    fn string_to_seek_after() {
        assert_eq!(
            Seek::try_from("YTphYmM").unwrap(),
            Seek::After("abc".into())
        );
    }

    #[test]
    fn string_to_seek_err() {
        assert!(Seek::try_from("$$$").is_err());
    }
}
