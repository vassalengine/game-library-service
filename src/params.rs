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

#[derive(Debug, Default, Deserialize)]
#[serde(try_from = "MaybeProjectsParams")]
pub struct ProjectsParams {
    pub seek: Seek,
    pub limit: Option<Limit>
}

// TODO: tests

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

impl TryFrom<MaybeProjectsParams> for ProjectsParams {
    type Error = Error;

    fn try_from(m: MaybeProjectsParams) -> Result<Self, Self::Error> {
        if (
                m.seek.is_some() &&
                (
                     m.sort.is_some() ||
                     m.order.is_some() ||
                     m.from.is_some() ||
                     m.q.is_some()
                )
            )
            ||
            (m.from.is_some() && m.q.is_some())
        {
            // sort, order, query, from are incompatible with seek
            // from is incompatible with query
            Err(Error::InvalidCombination(m))
        }
        else {
            let seek = match m.seek {
                Some(enc) => {
                    // base64-decode the seek string
                    let buf = base64::engine::general_purpose::URL_SAFE_NO_PAD
                        .decode(enc)?;

                    str::from_utf8(&buf)?
                        .parse::<Seek>()?
                },
                None => {
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

                    let dir = m.order.unwrap_or_else(
                            || sort_by.default_direction()
                    );

                    Seek { sort_by, dir, anchor }
                }
            };

            Ok(
                ProjectsParams {
                    seek,
                    limit: m.limit
                }
            )
        }
    }
}
