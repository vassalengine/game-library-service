use base64::{Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    str,
    num::NonZeroU8
};

use crate::errors::AppError;

// TODO: private fields various places

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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
#[serde(try_from = "&str")]
pub enum Anchor {
    Start,
    Before(String, u32),
    After(String, u32),
    StartQuery(String),
    BeforeQuery(String, f64, u32),
    AfterQuery(String, f64, u32)
}

// TODO: add Query type to SortBy, move it out of Anchor

impl From<&Anchor> for String {
    fn from(value: &Anchor) -> Self {
        match value {
            Anchor::Start => "s".into(),
            Anchor::Before(f, i) => format!("b:{}:{}", i, f),
            Anchor::After(f, i) => format!("a:{}:{}", i, f),
            Anchor::StartQuery(q) => format!("q:{}", q),
            Anchor::BeforeQuery(q, r, i) => format!("p:{}:{}:{}", i, r, q),
            Anchor::AfterQuery(q, r, i) => format!("r:{}:{}:{}", i, r, q)
        }
    }
}

impl TryFrom<&str> for Anchor {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut i = value.splitn(3, ':');

        let a = i.next().ok_or(AppError::MalformedQuery)?;

// TODO: StartQuery, BeforeQuery, AfterQuery
        match a {
            "s" => Ok(Anchor::Start),
            s => {
                let id = i.next()
                    .ok_or(AppError::MalformedQuery)?
                    .parse::<u32>()
                    .or(Err(AppError::MalformedQuery))?;
                let field = i.next()
                    .ok_or(AppError::MalformedQuery)?
                    .to_string();

                match s {
                    "b" => Ok(Anchor::Before(field, id)),
                    "a" => Ok(Anchor::After(field, id)),
                    _ => Err(AppError::MalformedQuery)
                }
            }
        }
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

// TODO: add tests for mtime, ctime

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub enum Direction {
    Ascending,
    Descending
}

impl Direction {
    pub fn rev(&self) -> Direction {
        match self {
            Direction::Ascending => Direction::Descending,
            Direction::Descending => Direction::Ascending
        }
    }
}

impl From<&Direction> for String {
    fn from(value: &Direction) -> Self {
        match value {
            Direction::Ascending => "a".into(),
            Direction::Descending => "d".into()
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "a" => Ok(Direction::Ascending),
            "d" => Ok(Direction::Descending),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl TryFrom<&u8> for Direction {
    type Error = AppError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            b'a' => Ok(Direction::Ascending),
            b'd' => Ok(Direction::Descending),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub enum SortBy {
    #[default]
    ProjectName,
    GameTitle,
    ModificationTime,
    CreationTime
}

impl From<&SortBy> for String {
    fn from(value: &SortBy) -> Self {
        match value {
            SortBy::ProjectName => "p".into(),
            SortBy::GameTitle => "t".into(),
            SortBy::ModificationTime => "m".into(),
            SortBy::CreationTime => "c".into()
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

impl TryFrom<&u8> for SortBy {
    type Error = AppError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            b'p' => Ok(SortBy::ProjectName),
            b't' => Ok(SortBy::GameTitle),
            b'm' => Ok(SortBy::ModificationTime),
            b'c' => Ok(SortBy::CreationTime),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl SortBy {
    pub fn default_direction(&self) -> Direction {
        match self {
            SortBy::ProjectName => Direction::Ascending,
            SortBy::GameTitle => Direction::Ascending,
            SortBy::ModificationTime => Direction::Descending,
            SortBy::CreationTime => Direction::Descending
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str")]
pub struct Seek {
    pub anchor: Anchor,
    pub sort_by: SortBy,
    pub dir: Direction
}

impl Default for Seek {
    fn default() -> Self {
        Seek {
            anchor: Anchor::Start,
            sort_by: SortBy::ProjectName,
            dir: Direction::Ascending
        }
    }
}

impl From<Seek> for String {
    fn from(value: Seek) -> Self {
        let s = format!("{}{}{}", value.sort_by, value.dir, value.anchor);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(s)
    }
}

impl TryFrom<&str> for Seek {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let buf = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| AppError::MalformedQuery)?;

        let mut i = buf.iter();

        let sort_by: SortBy = i.next()
            .ok_or(AppError::MalformedQuery)?
            .try_into()?;

        let dir: Direction = i.next()
            .ok_or(AppError::MalformedQuery)?
            .try_into()?;

        let anchor: Anchor = str::from_utf8(i.as_slice())
            .map_err(|_| AppError::MalformedQuery)?
            .try_into()?;

        Ok(Seek { sort_by, dir, anchor })
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
                    anchor: Anchor::Start,
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            ),
            "cGFz"
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
                    anchor: Anchor::Start,
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            ),
            "cGRz"
        );
    }

    #[test]
    fn seek_to_string_before() {
        assert_eq!(
            &String::from(
                Seek {
                    anchor: Anchor::Before("abc".into(), 0),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            ),
            "cGFiOjA6YWJj"
        );
    }

    #[test]
    fn seek_to_string_after() {
        assert_eq!(
            &String::from(
                Seek {
                    anchor: Anchor::After("abc".into(), 0),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            ),
            "cGFhOjA6YWJj"
        );
    }

    #[test]
    fn string_to_seek_start() {
        assert_eq!(
            Seek::try_from("cGFz").unwrap(),
            Seek {
                anchor: Anchor::Start,
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            }
        );
    }

    #[test]
    fn string_to_seek_end() {
        assert_eq!(
            Seek::try_from("cGRz").unwrap(),
            Seek {
                anchor: Anchor::Start,
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending
            }
        );
    }

    #[test]
    fn string_to_seek_before() {
        assert_eq!(
            Seek::try_from("cGFiOjA6YWJj").unwrap(),
            Seek {
                anchor: Anchor::Before("abc".into(), 0),
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            }
        );
    }

    #[test]
    fn string_to_seek_after() {
        assert_eq!(
            Seek::try_from("cGFhOjA6YWJj").unwrap(),
            Seek {
                anchor: Anchor::After("abc".into(), 0),
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending
            }
        );
    }

    #[test]
    fn string_to_seek_err() {
        assert!(Seek::try_from("$$$").is_err());
    }
}
