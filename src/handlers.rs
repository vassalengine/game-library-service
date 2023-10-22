use axum::{
    extract::{Path, State},
    response::Json
};
use std::sync::Arc;

use crate::{
    datastore::DataStore,
    errors::AppError,
    model::{Project, Projects, Owner, Users}
};

type DS = Arc<dyn DataStore + Send + Sync>;

pub async fn root_get() -> &'static str {
    "hello world"
}

pub async fn projects_get(
    State(_db): State<DS>
) -> Result<Json<Projects>, AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn project_get(
    Path(_proj_id): Path<u32>,
    State(_db): State<DS>
) -> Result<Json<Project>, AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn project_update(
//    _requester: Owner,
    Path(_proj_id): Path<u32>,
    State(_db): State<DS>
) -> Result<Json<Project>, AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn project_revision_get(
    Path(_proj_id): Path<u32>,
    Path(_revision): Path<u32>,
    State(_db): State<DS>
) -> Result<Json<Project>, AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn owners_get(
    Path(proj_id): Path<u32>,
    State(db): State<DS>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(db.get_owners(proj_id).await?))
}

pub async fn owners_add(
    _requester: Owner,
    Path(proj_id): Path<u32>,
    State(db): State<DS>,
    Json(owners): Json<Users>
) -> Result<(), AppError> {
    Ok(db.add_owners(&owners, proj_id).await?)
}

pub async fn owners_remove(
    _requester: Owner,
    Path(proj_id): Path<u32>,
    State(db): State<DS>,
    Json(owners): Json<Users>
) -> Result<(), AppError>
{
    Ok(db.remove_owners(&owners, proj_id).await?)
}

pub async fn players_get(
    Path(_proj_id): Path<u32>,
    State(_db): State<DS>
) -> Result<Json<Users>, AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn players_add(
//    requester: Player,
    Path(_proj_id): Path<u32>,
    State(_db): State<DS>
) -> Result<(), AppError> {
    Err(AppError::NotImplemented)
}

pub async fn players_remove(
//    requester: Player,
    Path(_proj_id): Path<u32>,
    State(_db): State<DS>,
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn package_get(
    Path(_proj_id): Path<u32>,
    Path(_pkg_name): Path<String>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn package_version_get(
    Path(_proj_id): Path<u32>,
    Path(_pkg_name): Path<String>,
    Path(_pkg_version): Path<String>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn package_version_put(
    Path(_proj_id): Path<u32>,
    Path(_pkg_name): Path<String>,
    Path(_pkg_version): Path<String>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn readme_get(
    Path(_proj_id): Path<u32>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn readme_revision_get(
    Path(_proj_id): Path<u32>,
    Path(_revision): Path<u32>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn image_get(
    Path(_proj_id): Path<u32>,
    Path(_img_name): Path<String>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

pub async fn image_put(
    Path(_proj_id): Path<u32>,
    Path(_img_name): Path<String>,
    State(_db): State<DS>
) -> Result<(), AppError>
{
    Err(AppError::NotImplemented)
}

#[cfg(test)]
mod test {
    use super::*;

/*
    #[tokio::test]
    async fn root_ok() {

    }
*/
}
