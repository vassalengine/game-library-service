use axum::{
    extract::{Path, Query, State},
    response::{Json, Redirect}
};

use crate::{
    core::CoreArc,
    errors::AppError,
    model::{Owned, OwnedOrNew, PackageID, Packages, ProjectData, ProjectDataPut, ProjectID, Projects, Readme, Users, User},
    pagination::{Limit, Seek, PaginationParams}
};

pub async fn root_get() -> &'static str {
    "hello world"
}

pub async fn projects_get(
    Query(params): Query<PaginationParams>,
    State(core): State<CoreArc>
) -> Result<Json<Projects>, AppError>
{
    Ok(
        Json(
            core.get_projects(
                params.seek.unwrap_or(Seek::Start),
                params.limit.unwrap_or(Limit::DEFAULT)
            ).await?
        )
    )
}

pub async fn project_get(
    proj_id: ProjectID,
    State(core): State<CoreArc>
) -> Result<Json<ProjectData>, AppError>
{
    Ok(Json(core.get_project(proj_id.0).await?))
}

pub async fn project_put(
    owned: OwnedOrNew,
    Path(proj): Path<String>,
    State(core): State<CoreArc>,
    Json(proj_data): Json<ProjectDataPut>
) -> Result<(), AppError>
{
    match owned {
        OwnedOrNew::Owned(owned) => {
            core.update_project(owned.1.0, &proj_data).await
        },
        OwnedOrNew::User(user) => {
            core.create_project(&user, &proj, &proj_data).await
        }
    }
}

pub async fn project_revision_get(
    proj_id: ProjectID,
    Path((_, revision)): Path<(String, u32)>,
    State(core): State<CoreArc>
) -> Result<Json<ProjectData>, AppError>
{
    Ok(Json(core.get_project_revision(proj_id.0, revision).await?))
}

pub async fn owners_get(
    proj_id: ProjectID,
    State(core): State<CoreArc>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_owners(proj_id.0).await?))
}

pub async fn owners_add(
    Owned(_, proj_id): Owned,
    State(core): State<CoreArc>,
    Json(owners): Json<Users>
) -> Result<(), AppError>
{
    core.add_owners(&owners, proj_id.0).await
}

pub async fn owners_remove(
    Owned(_, proj_id): Owned,
    State(core): State<CoreArc>,
    Json(owners): Json<Users>
) -> Result<(), AppError>
{
    core.remove_owners(&owners, proj_id.0).await
}

pub async fn players_get(
    proj_id: ProjectID,
    State(core): State<CoreArc>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_players(proj_id.0).await?))
}

pub async fn players_add(
    requester: User,
    proj_id: ProjectID,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    core.add_player(&requester, proj_id.0).await
}

pub async fn players_remove(
    requester: User,
    proj_id: ProjectID,
    State(core): State<CoreArc>,
) -> Result<(), AppError>
{
    core.remove_player(&requester, proj_id.0).await
}

pub async fn packages_get(
    proj_id: ProjectID,
    State(core): State<CoreArc>
) -> Result<Json<Packages>, AppError>
{
    Ok(Json(core.get_packages(proj_id.0).await?))
}

pub async fn package_get(
    proj_id: ProjectID,
    pkg_id: PackageID,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_package(proj_id.0, pkg_id.0).await?))
}

pub async fn package_version_get(
    proj_id: ProjectID,
    pkg_id: PackageID,
    Path((_, _, pkg_version)): Path<(String, String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(
        Redirect::to(
            &core.get_package_version(proj_id.0, pkg_id.0, &pkg_version).await?
        )
    )
}

pub async fn package_version_put(
    _proj_id: ProjectID,
    Path((_pkg_name, _pkg_version)): Path<(String, String)>,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn readme_get(
    proj_id: ProjectID,
    State(core): State<CoreArc>
) -> Result<Json<Readme>, AppError>
{
    Ok(Json(core.get_readme(proj_id.0).await?))
}

pub async fn readme_revision_get(
    proj_id: ProjectID,
    Path((_, revision)): Path<(String, u32)>,
    State(core): State<CoreArc>
) -> Result<Json<Readme>, AppError>
{
    Ok(Json(core.get_readme_revision(proj_id.0, revision).await?))
}

pub async fn image_get(
    _proj_id: ProjectID,
    Path(_img_name): Path<String>,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn image_put(
    _proj_id: ProjectID,
    Path(_img_name): Path<String>,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn flag_post(
    _requester: User,
    _proj_id: ProjectID,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}
