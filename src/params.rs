use base64::{Engine as _};
use serde::Deserialize;
use std::str;

use crate::pagination::{Anchor, Limit, Direction, SortBy, Seek, SeekError};

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
pub struct MaybeProjectsParams {
    pub q: Option<String>,
    pub sort: Option<SortBy>,
    pub order: Option<Direction>,
    pub from: Option<String>,
    pub seek: Option<String>,
    pub limit: Option<Limit>
}

impl MaybeProjectsParams {
    fn valid(&self) -> bool {
        // sort, order, query, from are incompatible with seek
        // from is incompatible with query
        !(
            (
                self.seek.is_some() &&
                (
                    self.sort.is_some() ||
                    self.order.is_some() ||
                    self.from.is_some() ||
                    self.q.is_some()
                )
            )
            ||
            (self.from.is_some() && self.q.is_some())
        )
    }
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(try_from = "MaybeProjectsParams")]
pub struct ProjectsParams {
    pub seek: Seek,
    pub limit: Option<Limit>
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("invalid combination {0:?}")]
    InvalidCombination(MaybeProjectsParams),
    #[error("invalid base64 {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("invalid UTF-8 {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    SeekError(#[from] SeekError)
}

fn decode_seek(enc: &str) -> Result<Seek, Error> {
    // base64-decode the seek string
    let buf = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(enc)?;

    Ok(
        str::from_utf8(&buf)?
            .parse::<Seek>()?
    )
}

fn convert_non_seek(m: MaybeProjectsParams) -> Seek {
    let (sort_by, anchor) = match m.q {
        Some(query) => (
            m.sort.unwrap_or(SortBy::Relevance),
            Anchor::StartQuery(query)
        ),
        None => (
            m.sort.unwrap_or_default(),
            match m.from {
                // id 0 is unused and sorts before all other
                // instances of the from string
                Some(from) => Anchor::After(from, 0),
                None => Anchor::Start
            }
        )
    };

    let dir = m.order.unwrap_or_else(|| sort_by.default_direction());

    Seek { sort_by, dir, anchor }
}

impl TryFrom<MaybeProjectsParams> for ProjectsParams {
    type Error = Error;

    fn try_from(m: MaybeProjectsParams) -> Result<Self, Self::Error> {
        match m.valid() {
            true => Ok(
                ProjectsParams {
                    limit: m.limit,
                    seek: match m.seek {
                        Some(enc) => decode_seek(&enc)?,
                        None => convert_non_seek(m)
                    }
                }
            ),
            false => Err(Error::InvalidCombination(m))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maybe_projects_params_valid() {
        let mpp = MaybeProjectsParams {
            sort: Some(SortBy::ProjectName),
            order: Some(Direction::Ascending),
            ..Default::default()
        };
        assert!(mpp.valid());
    }

    #[test]
    fn maybe_projects_params_invald_seek_and_sort() {
        let mpp = MaybeProjectsParams {
            seek: Some("whatever".into()),
            sort: Some(SortBy::ProjectName),
            ..Default::default()
        };
        assert!(!mpp.valid());
    }

    #[test]
    fn maybe_projects_params_invalid_seek_and_order() {
        let mpp = MaybeProjectsParams {
            seek: Some("whatever".into()),
            order: Some(Direction::Ascending),
            ..Default::default()
        };
        assert!(!mpp.valid());
    }

    #[test]
    fn maybe_projects_params_invalid_seek_and_from() {
        let mpp = MaybeProjectsParams {
            seek: Some("whatever".into()),
            from: Some("whatever".into()),
            ..Default::default()
        };
        assert!(!mpp.valid());
    }

    #[test]
    fn maybe_projects_params_invalid_seek_and_q() {
        let mpp = MaybeProjectsParams {
            seek: Some("whatever".into()),
            q: Some("whatever".into()),
            ..Default::default()
        };
        assert!(!mpp.valid());
    }

    #[test]
    fn maybe_projects_params_invalid_from_and_q() {
        let mpp = MaybeProjectsParams {
            from: Some("whatever".into()),
            q: Some("whatever".into()),
            ..Default::default()
        };
        assert!(!mpp.valid());
    }

    #[test]
    fn decode_seek_ok() {
        assert_eq!(
            decode_seek("cCxhLGEsYWJjLCww").unwrap(),
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::After("abc".into(), 0)
            }
        );
    }

    #[test]
    fn decode_seek_bad_base64() {
        assert!(
            matches!(
                decode_seek("garbage!!!").unwrap_err(),
                Error::Base64DecodeError(_)
            )
        );
    }

    #[test]
    fn decode_seek_bad_utf8() {
        // ____ decodes to FF FF FF, which is not valid UTF-8
        assert!(
            matches!(
                decode_seek("____").unwrap_err(),
                Error::Utf8Error(_)
            )
        );
    }

    #[test]
    fn maybe_projects_params_try_from_ok() {
        let mpp = MaybeProjectsParams {
            sort: Some(SortBy::ProjectName),
            order: Some(Direction::Ascending),
            ..Default::default()
        };

        let pp = ProjectsParams {
            seek: Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::Start
            },
            limit: None
        };

        assert_eq!(ProjectsParams::try_from(mpp).unwrap(), pp);
    }

    #[test]
    fn maybe_projects_params_try_from_invalid() {
        let mpp = MaybeProjectsParams {
            seek: Some("whatever".into()),
            sort: Some(SortBy::ProjectName),
            ..Default::default()
        };

        assert!(
            matches!(
                ProjectsParams::try_from(mpp).unwrap_err(),
                Error::InvalidCombination(_)
            )
        );
    }

    #[test]
    fn maybe_projects_params_try_from_bad_base64() {
        let mpp = MaybeProjectsParams {
            seek: Some("garbage!!!".into()),
            ..Default::default()
        };

        assert!(
            matches!(
                ProjectsParams::try_from(mpp).unwrap_err(),
                Error::Base64DecodeError(_)
            )
        );
    }

    #[test]
    fn maybe_projects_params_try_from_bad_utf8() {
        // _v7- decodes to FE FE FE, which is not valid UTF-8
        let mpp = MaybeProjectsParams {
            seek: Some("_v7-".into()),
            ..Default::default()
        };

        assert!(
            matches!(
                ProjectsParams::try_from(mpp).unwrap_err(),
                Error::Utf8Error(_)
            )
        );
    }
}
