use serde::Deserialize;

use crate::{
    errors::AppError,
    pagination::{Anchor, Limit, Direction, SortBy, Seek}
};

#[derive(Debug, Default, Deserialize)]
pub struct MaybeProjectsParams {
    pub q: Option<String>,
    pub sort: Option<SortBy>,
    pub order: Option<Direction>,
    pub seek: Option<Seek>,
    pub limit: Option<Limit>
}

#[derive(Debug, Default, Deserialize)]
#[serde(try_from = "MaybeProjectsParams")]
pub struct ProjectsParams {
    pub q: Option<String>,
    pub seek: Seek,
    pub limit: Limit
}

impl TryFrom<MaybeProjectsParams> for ProjectsParams {
    type Error = AppError;

    fn try_from(m: MaybeProjectsParams) -> Result<Self, Self::Error> {
        if (m.sort.is_some() || m.order.is_some()) && m.seek.is_some() {
            // sort, order are incompatible with seek
            Err(AppError::MalformedQuery)
        }
        else {
            let seek = match m.seek {
                Some(seek) => seek,
                None => {
                    // convert sort params into seek params
                    let sort_by = m.sort.unwrap_or_default();
                    let dir = m.order.unwrap_or_else(
                        || sort_by.default_direction()
                    );
                    Seek {
                        sort_by,
                        dir,
                        anchor: Anchor::Start
                    }
                }
            };

            Ok(
                ProjectsParams {
                    q: m.q,
                    limit: m.limit.unwrap_or_default(),
                    seek
                }
            )
        }
    }
}
