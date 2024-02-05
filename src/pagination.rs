use base64::{Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    str::{self, FromStr},
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

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(try_from = "&str", into = "String")]
enum AnchorTag {
    Start,
    Before,
    After,
    StartQuery,
    BeforeQuery,
    AfterQuery
}

impl From<AnchorTag> for String {
    fn from(value: AnchorTag) -> Self {
        match value {
            AnchorTag::Start => "s".into(),
            AnchorTag::Before => "b".into(),
            AnchorTag::After => "a".into(),
            AnchorTag::StartQuery => "q".into(),
            AnchorTag::BeforeQuery => "p".into(),
            AnchorTag::AfterQuery => "r".into()
        }
    }
}

impl TryFrom<&str> for AnchorTag {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "s" => Ok(AnchorTag::Start),
            "b" => Ok(AnchorTag::Before),
            "a" => Ok(AnchorTag::After),
            "q" => Ok(AnchorTag::StartQuery),
            "p" => Ok(AnchorTag::BeforeQuery),
            "r" => Ok(AnchorTag::AfterQuery),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RawAnchor {
    tag: AnchorTag,
    field: Option<String>,
    query: Option<String>,
    rank: Option<f64>,
    id: Option<u32>
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "RawAnchor", into = "RawAnchor")]
pub enum Anchor {
    Start,
    Before(String, u32),
    After(String, u32),
    StartQuery(String),
    BeforeQuery(String, f64, u32),
    AfterQuery(String, f64, u32)
}

impl TryFrom<RawAnchor> for Anchor {
    type Error = AppError;

    fn try_from(ra: RawAnchor) -> Result<Self, Self::Error> {
        match (ra.tag, ra.field, ra.query, ra.rank, ra.id) {
            (AnchorTag::Start, None, None, None, None) => Ok(Anchor::Start),
            (AnchorTag::Before, Some(f), None, None, Some(i)) => Ok(Anchor::Before(f, i)),
            (AnchorTag::After, Some(f), None, None, Some(i)) => Ok(Anchor::After(f, i)),
            (AnchorTag::StartQuery, None, Some(q), None, None) => Ok(Anchor::StartQuery(q)),
            (AnchorTag::BeforeQuery, None, Some(q), Some(r), Some(i)) => Ok(Anchor::BeforeQuery(q, r, i)),
            (AnchorTag::AfterQuery, None, Some(q), Some(r), Some(i)) => Ok(Anchor::AfterQuery(q, r, i)),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl From<Anchor> for RawAnchor {
    fn from(a: Anchor) -> Self {
        match a {
            Anchor::Start => RawAnchor {
                tag: AnchorTag::Start,
                field: None,
                query: None,
                rank: None,
                id: None
            },
            Anchor::Before(field, id) => RawAnchor {
                tag: AnchorTag::Before,
                field: Some(field),
                query: None,
                rank: None,
                id: Some(id)
            },
            Anchor::After(field, id) => RawAnchor {
                tag: AnchorTag::After,
                field: Some(field),
                query: None,
                rank: None,
                id: Some(id)
            },
            Anchor::StartQuery(query) => RawAnchor {
                tag: AnchorTag::StartQuery,
                field: None,
                query: Some(query),
                rank: None,
                id: None
            },
            Anchor::BeforeQuery(query, rank, id) => RawAnchor {
                tag: AnchorTag::BeforeQuery,
                field: None,
                query: Some(query),
                rank: Some(rank),
                id: Some(id)
            },
            Anchor::AfterQuery(query, rank, id) => RawAnchor {
                tag: AnchorTag::AfterQuery,
                field: None,
                query: Some(query),
                rank: Some(rank),
                id: Some(id)
            }
        }
    }
}

// TODO: add tests for mtime, ctime

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str", into = "String")]
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

impl From<Direction> for String {
    fn from(value: Direction) -> Self {
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

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "&str", into = "String")]
pub enum SortBy {
    ProjectName,
    #[default]
    GameTitle,
    ModificationTime,
    CreationTime,
    Relevance
}

impl From<SortBy> for String {
    fn from(value: SortBy) -> Self {
        match value {
            SortBy::ProjectName => "p".into(),
            SortBy::GameTitle => "t".into(),
            SortBy::ModificationTime => "m".into(),
            SortBy::CreationTime => "c".into(),
            SortBy::Relevance => "r".into()
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
            "r" => Ok(SortBy::Relevance),
            _ => Err(AppError::MalformedQuery)
        }
    }
}

impl SortBy {
    pub fn default_direction(&self) -> Direction {
        match self {
            SortBy::ProjectName => Direction::Ascending,
            SortBy::GameTitle => Direction::Ascending,
            SortBy::ModificationTime => Direction::Descending,
            SortBy::CreationTime => Direction::Descending,
            SortBy::Relevance => Direction::Ascending
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Seek {
    pub sort_by: SortBy,
    pub dir: Direction,
    pub anchor: Anchor
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

impl TryFrom<Seek> for String {
    type Error = AppError;

    fn try_from(s: Seek) -> Result<Self, Self::Error> {
        String::try_from(&s)
    }
}

impl TryFrom<&Seek> for String {
    type Error = AppError;

    fn try_from(s: &Seek) -> Result<Self, Self::Error> {
        let mut w = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(vec![]);

        w.serialize(s).map_err(|_| AppError::MalformedQuery)?;
        let mut b = w.into_inner().map_err(|_| AppError::MalformedQuery)?;
        b.pop(); // drop the terminator
        String::from_utf8(b).map_err(|_| AppError::MalformedQuery)
    }
}

impl FromStr for Seek {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(s.as_bytes());

        if let Some(result) = r.deserialize().next() {
            let seek: Seek = result.map_err(|_| AppError::MalformedQuery)?;

            // Relevance must be paired with StartQuery, AfterQuery, BeforeQuery
            // Other SortBy must be paired with Start, After, Before
            match seek.sort_by {
                SortBy::Relevance => match seek.anchor {
                    Anchor::StartQuery(..) |
                    Anchor::AfterQuery(..) |
                    Anchor::BeforeQuery(..) => Ok(seek),
                    _ => Err(AppError::MalformedQuery)
                },
                _ => match seek.anchor {
                    Anchor::Start |
                    Anchor::After(..) |
                    Anchor::Before(..) => Ok(seek),
                    _ => Err(AppError::MalformedQuery),
                }
            }
        }
        else {
            Err(AppError::MalformedQuery)
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SeekLink(String);

impl SeekLink {
    pub fn new(seek: &Seek, limit: Option<Limit>) -> Result<SeekLink, AppError> {
        let s = String::try_from(seek)
            .map_err(|_| AppError::MalformedQuery)?;
        let s = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(s);

        match limit {
            Some(l) => Ok(SeekLink(format!("?limit={}&seek={}", l, s))),
            None => Ok(SeekLink(format!("?seek={}", s)))
        }
    }
}

impl fmt::Display for SeekLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Pagination {
    pub prev_page: Option<SeekLink>,
    pub next_page: Option<SeekLink>,
    pub total: i64
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
    fn seek_roundtrip_start() {
        let seek = Seek {
            sort_by: SortBy::ProjectName,
            dir: Direction::Ascending,
            anchor: Anchor::Start,
        };

        assert_eq!(
            String::try_from(&seek)
                .unwrap()
                .parse::<Seek>()
                .unwrap(),
            seek
        );
    }

    #[test]
    fn seek_to_string_start() {
        assert_eq!(
            &String::try_from(
                Seek {
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending,
                    anchor: Anchor::Start
                }
            ).unwrap(),
            "p,a,s,,,,"
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
            &String::try_from(
                Seek {
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending,
                    anchor: Anchor::Start
                }
            ).unwrap(),
            "p,d,s,,,,"
        );
    }

    #[test]
    fn seek_to_string_before() {
        assert_eq!(
            &String::try_from(
                Seek {
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending,
                    anchor: Anchor::Before("abc".into(), 0),
                }
            ).unwrap(),
            "p,a,b,abc,,,0"
        );
    }

    #[test]
    fn seek_to_string_after() {
        assert_eq!(
            &String::try_from(
                Seek {
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending,
                    anchor: Anchor::After("abc".into(), 0)
                }
            ).unwrap(),
            "p,a,a,abc,,,0"
        );
    }

    #[test]
    fn string_to_seek_start() {
        assert_eq!(
            "p,a,s,,,,".parse::<Seek>().unwrap(),
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::Start
            }
        );
    }

    #[test]
    fn string_to_seek_end() {
        assert_eq!(
            "p,d,s,,,,".parse::<Seek>().unwrap(),
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::Start
            }
        );
    }

    #[test]
    fn string_to_seek_before() {
        assert_eq!(
            "p,a,b,abc,,,0".parse::<Seek>().unwrap(),
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::Before("abc".into(), 0)
            }
        );
    }

    #[test]
    fn string_to_seek_after() {
        assert_eq!(
            "p,a,a,abc,,,0".parse::<Seek>().unwrap(),
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::After("abc".into(), 0)
            }
        );
    }

    #[test]
    fn string_to_seek_err() {
        assert!("$$$".parse::<Seek>().is_err());
    }
}
