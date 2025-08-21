use serde::Deserialize;
use std::str;

use crate::pagination::{Anchor, Limit, Direction, SortBy, Seek, SeekError};

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
pub struct MaybeProjectsParams {
    pub q: Option<String>,
    pub from: Option<String>,
    pub sort_by: Option<SortBy>,
    pub dir: Option<Direction>,
    pub anchor: Option<Anchor>,
    pub limit: Option<Limit>,
//    pub facets: Option<Vec<Facet>>
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(try_from = "MaybeProjectsParams")]
pub struct ProjectsParams {
/*
    pub sort_by: Option<SortBy>,
    pub dir: Option<Direction>,
    pub anchor: Option<Anchor>,
*/
    pub seek: Seek,
    pub limit: Option<Limit>
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("invalid combination {0:?}")]
    InvalidCombination(MaybeProjectsParams),
    #[error("{0}")]
    Seek(#[from] SeekError)
}

impl TryFrom<MaybeProjectsParams> for Seek {
    type Error = Error;

    fn try_from(m: MaybeProjectsParams) -> Result<Self, Self::Error> {
        match m {
            // all seek parts, nothing else
            MaybeProjectsParams {
                q: None,
                from: None,
                sort_by: Some(sort_by),
                dir: Some(dir),
                anchor: Some(anchor),
                ..
            } => Ok((sort_by, dir, anchor).try_into()?),
            // query with optional sort_by, dir
            MaybeProjectsParams {
                q: Some(query),
                from: None,
                sort_by,
                dir,
                anchor: None,
                ..
            } => {
                let sort_by = sort_by.unwrap_or(SortBy::Relevance);
                let dir = dir.unwrap_or_else(|| sort_by.default_direction());
                Ok((sort_by, dir, Anchor::StartQuery(query)).try_into()?)
            },
            // no query; optional sort_by, dir, from
            MaybeProjectsParams {
                q: None,
                from,
                sort_by,
                dir,
                anchor: None,
                ..
            } => {
                let sort_by = sort_by.unwrap_or_default();
                let dir = dir.unwrap_or_else(|| sort_by.default_direction());
                Ok((
                    sort_by,
                    dir,
                    match from {
                        // id 0 is unused and sorts before all other
                        // instances of the from string
                        Some(from) => Anchor::After(from, 0),
                        None => Anchor::Start
                    }
                ).try_into()?)
            },
            _ => Err(Error::InvalidCombination(m))
        }
    }
}

impl TryFrom<MaybeProjectsParams> for ProjectsParams {
    type Error = Error;

    fn try_from(m: MaybeProjectsParams) -> Result<Self, Self::Error> {
        Ok(
            ProjectsParams {
                limit: m.limit,
                seek: m.try_into()?
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maybe_projects_params_default_ok() {
        let mpp = MaybeProjectsParams::default();

        let pp = ProjectsParams {
            seek: Seek {
                sort_by: SortBy::default(),
                dir: SortBy::default().default_direction(),
                anchor: Anchor::Start
            },
            limit: None
        };

        assert_eq!(ProjectsParams::try_from(mpp).unwrap(), pp);
    }

    #[test]
    fn maybe_projects_params_from_and_q_invalid() {
        let mpp = MaybeProjectsParams {
            from: Some("whatever".into()),
            q: Some("whatever".into()),
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
    fn maybe_projects_params_seek_ok() {
        let mpp = MaybeProjectsParams {
            sort_by: Some(SortBy::ProjectName),
            dir: Some(Direction::Ascending),
            anchor: Some(Anchor::Start),
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
    fn maybe_projects_params_seek_incomplete() {
        let mpp = MaybeProjectsParams {
            sort_by: Some(SortBy::ProjectName),
            anchor: Some(Anchor::Start),
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
    fn maybe_projects_params_mixed_invalid() {
        let mpp = MaybeProjectsParams {
            sort_by: Some(SortBy::ProjectName),
            anchor: Some(Anchor::Start),
            dir: Some(Direction::Ascending),
            from: Some("whatever".into()),
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
    fn maybe_projects_params_sort_by_dir_ok() {
        let mpp = MaybeProjectsParams {
            sort_by: Some(SortBy::ProjectName),
            dir: Some(Direction::Ascending),
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
    fn maybe_projects_params_from_ok() {
        let mpp = MaybeProjectsParams {
            from: Some("whatever".into()),
            ..Default::default()
        };

        let pp = ProjectsParams {
            seek: Seek {
                sort_by: SortBy::default(),
                dir: SortBy::default().default_direction(),
                anchor: Anchor::After("whatever".into(), 0)
            },
            limit: None
        };

        assert_eq!(ProjectsParams::try_from(mpp).unwrap(), pp);
    }

    #[test]
    fn maybe_projects_params_limit_ok() {
        let mpp = MaybeProjectsParams {
            limit: Limit::new(50),
            ..Default::default()
        };

        let pp = ProjectsParams {
            seek: Seek {
                sort_by: SortBy::default(),
                dir: SortBy::default().default_direction(),
                anchor: Anchor::Start
            },
            limit: Limit::new(50)
        };

        assert_eq!(ProjectsParams::try_from(mpp).unwrap(), pp);
    }
}
