use base64::{Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
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

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum OrderDirection {
    Ascending,
    Descending
}

impl From<&OrderDirection> for String {
    fn from(value: &OrderDirection) -> Self {
        match value {
            OrderDirection::Ascending => "a".to_string(),
            OrderDirection::Descending => "d".to_string()
        }
    }
}

impl TryFrom<&str> for OrderDirection {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "a" => Ok(OrderDirection::Ascending),
            "d" => Ok(OrderDirection::Descending),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub enum Anchor {
    Start(OrderDirection),
    Before(OrderDirection, String, u32),
    After(OrderDirection, String, u32),
    End(OrderDirection)
}

impl From<Anchor> for String {
    fn from(value: Anchor) -> Self {
        match value {
            Anchor::Start(d) => format!("s:{}", d),
            Anchor::Before(d, n, i) => format!("b:{}:{}:{}", d, i, n),
            Anchor::After(d, n, i) => format!("a:{}:{}:{}", d, i, n),
            Anchor::End(d) =>  format!("e:{}", d)
        }
    }
}

impl TryFrom<&str> for Anchor {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut i = value.splitn(4, ':');

        let a = i.next().ok_or(AppError::MalformedQuery)?;
        let d = OrderDirection::try_from(
            i.next()
                .ok_or(AppError::MalformedQuery)?
        )?;

        match a {
            "s" => Ok(Anchor::Start(d)),
            "e" => Ok(Anchor::End(d)),
            s => {
                let id = i.next()
                    .ok_or(AppError::MalformedQuery)?
                    .parse::<u32>()
                    .or(Err(AppError::MalformedQuery))?;
                let name = i.next()
                    .ok_or(AppError::MalformedQuery)?
                    .to_string();

                match s {
                    "b" => Ok(Anchor::Before(d, name, id)),
                    "a" => Ok(Anchor::After(d, name, id)),
                    _ => Err(AppError::MalformedQuery)
                }
            }
        }
    }
}

// TODO: add tests for mtime, ctime

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub enum SortBy {
    ProjectName,
    GameTitle,
    ModificationTime,
    CreationTime
}

impl From<SortBy> for String {
    fn from(value: SortBy) -> Self {
        match value {
            SortBy::ProjectName => "p".to_string(),
            SortBy::GameTitle => "t".to_string(),
            SortBy::ModificationTime => "m".to_string(),
            SortBy::CreationTime => "c".to_string()
        }
    }
}

impl TryFrom<&str> for SortBy {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "p" => Ok(SortBy::ProjectName),
            "t" => Ok(SortBy::GameTitle),
            "m" => Ok(SortBy::ModificationTime),
            "c" => Ok(SortBy::CreationTime),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl SortBy {
    pub fn default_direction(&self) -> OrderDirection {
        match self {
            SortBy::ProjectName => OrderDirection::Ascending,
            SortBy::GameTitle => OrderDirection::Ascending,
            SortBy::ModificationTime => OrderDirection::Descending,
            SortBy::CreationTime => OrderDirection::Descending
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub struct Seek {
    pub anchor: Anchor,
    pub sort_by: SortBy
}

impl From<Seek> for String {
    fn from(value: Seek) -> Self {
        let s = String::from(value.sort_by) + &String::from(value.anchor);
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

        let mut i = d.chars();
        let c = i.next().ok_or(AppError::MalformedQuery)?.to_string();
        let sort_by = SortBy::try_from(c.as_str())?;
        let anchor = Anchor::try_from(i.as_str())?;

        Ok(
            Seek {
                anchor,
                sort_by
            }
        )
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SeekLink(String);

impl SeekLink {
    pub fn new(seek: Seek) -> SeekLink {
        SeekLink("?seek=".to_string() + &String::from(seek))
    }
}

impl From<Seek> for SeekLink {
    fn from(seek: Seek) -> Self {
        SeekLink::new(seek)
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
            &String::from(
                Seek {
                    anchor: Anchor::Start(OrderDirection::Ascending),
                    sort_by: SortBy::ProjectName
                }
            ),
            "cHM6YQ"
        );
    }

/*
    #[test]
    fn xxx() {
        assert_eq!(
            &String::from(
                Seek {
                    anchor: Anchor::Start,
                    sort_by: SortBy::GameTitle
                }
            ),
            "cHM6"
        );
    }
*/

    #[test]
    fn seek_to_string_end() {
        assert_eq!(
            &String::from(
                Seek {
                    anchor: Anchor::End(OrderDirection::Ascending),
                    sort_by: SortBy::ProjectName
                }
            ),
            "cGU6YQ"
        );
    }

    #[test]
    fn seek_to_string_before() {
        assert_eq!(
            &String::from(
                Seek {
                    anchor: Anchor::Before(OrderDirection::Ascending, "abc".into(), 0),
                    sort_by: SortBy::ProjectName
                }
            ),
            "cGI6YTowOmFiYw"
        );
    }

    #[test]
    fn seek_to_string_after() {
        assert_eq!(
            &String::from(
                Seek {
                    anchor: Anchor::After(OrderDirection::Ascending, "abc".into(), 0),
                    sort_by: SortBy::ProjectName
                }
            ),
            "cGE6YTowOmFiYw"
        );
    }

    #[test]
    fn string_to_seek_start() {
        assert_eq!(
            Seek::try_from("cHM6YQ").unwrap(),
            Seek {
                anchor: Anchor::Start(OrderDirection::Ascending),
                sort_by: SortBy::ProjectName
            }
        );
    }

    #[test]
    fn string_to_seek_end() {
        assert_eq!(
            Seek::try_from("cGU6YQ").unwrap(),
            Seek {
                anchor: Anchor::End(OrderDirection::Ascending),
                sort_by: SortBy::ProjectName
            }
        );
    }

    #[test]
    fn string_to_seek_before() {
        assert_eq!(
            Seek::try_from("cGI6YTowOmFiYw").unwrap(),
            Seek {
                anchor: Anchor::Before(OrderDirection::Ascending, "abc".into(), 0),
                sort_by: SortBy::ProjectName
            }
        );
    }

    #[test]
    fn string_to_seek_after() {
        assert_eq!(
            Seek::try_from("cGE6YTowOmFiYw").unwrap(),
            Seek {
                anchor: Anchor::After(OrderDirection::Ascending, "abc".into(), 0),
                sort_by: SortBy::ProjectName
            }
        );
    }

    #[test]
    fn string_to_seek_err() {
        assert!(Seek::try_from("$$$").is_err());
    }
}
