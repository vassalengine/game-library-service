use axum::{
    extract::{Path, State},
    response::Json
};
use std::sync::Arc;

use crate::{
    core::Core,
    errors::AppError,
    model::{Project, Projects, Owner, Users, User}
};

type CS = Arc<dyn Core + Send + Sync>;

pub async fn root_get() -> &'static str {
    "hello world"
}

pub async fn projects_get(
    State(_core): State<CS>
) -> Result<Json<Projects>, AppError>
{
    todo!();
}

pub async fn project_get(
    Path(_proj_id): Path<u32>,
    State(_core): State<CS>
) -> Result<Json<Project>, AppError>
{
    todo!();
}

pub async fn project_update(
//    _requester: Owner,
    Path(_proj_id): Path<u32>,
    State(_core): State<CS>
) -> Result<Json<Project>, AppError>
{
    todo!();
}

pub async fn project_revision_get(
    Path(_proj_id): Path<u32>,
    Path(_revision): Path<u32>,
    State(_core): State<CS>
) -> Result<Json<Project>, AppError>
{
    todo!();
}

pub async fn owners_get(
    Path(proj_id): Path<u32>,
    State(core): State<CS>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_owners(proj_id).await?))
}

pub async fn owners_add(
    _: Owner,
    Path(proj_id): Path<u32>,
    State(core): State<CS>,
    Json(owners): Json<Users>
) -> Result<(), AppError>
{
    core.add_owners(&owners, proj_id).await
}

pub async fn owners_remove(
    _: Owner,
    Path(proj_id): Path<u32>,
    State(core): State<CS>,
    Json(owners): Json<Users>
) -> Result<(), AppError>
{
    core.remove_owners(&owners, proj_id).await
}

pub async fn players_get(
    Path(proj_id): Path<u32>,
    State(core): State<CS>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_players(proj_id).await?))
}

pub async fn players_add(
    requester: User,
    Path(proj_id): Path<u32>,
    State(core): State<CS>
) -> Result<(), AppError>
{
    core.add_player(&requester, proj_id).await
}

pub async fn players_remove(
    requester: User,
    Path(proj_id): Path<u32>,
    State(core): State<CS>,
) -> Result<(), AppError>
{
    core.remove_player(&requester, proj_id).await
}

pub async fn package_get(
    Path(_proj_id): Path<u32>,
    Path(_pkg_name): Path<String>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn package_version_get(
    Path(_proj_id): Path<u32>,
    Path(_pkg_name): Path<String>,
    Path(_pkg_version): Path<String>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn package_version_put(
    Path(_proj_id): Path<u32>,
    Path(_pkg_name): Path<String>,
    Path(_pkg_version): Path<String>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn readme_get(
    Path(_proj_id): Path<u32>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn readme_revision_get(
    Path(_proj_id): Path<u32>,
    Path(_revision): Path<u32>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn image_get(
    Path(_proj_id): Path<u32>,
    Path(_img_name): Path<String>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn image_put(
    Path(_proj_id): Path<u32>,
    Path(_img_name): Path<String>,
    State(_core): State<CS>
) -> Result<(), AppError>
{
    todo!();
}
