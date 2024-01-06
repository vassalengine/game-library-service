use axum::{
    extract::{Path, Query, State},
    response::{Json, Redirect}
};

use crate::{
    core::CoreArc,
    errors::AppError,
    extractors::Wrapper,
    model::{Owned, OwnedOrNew, PackageID, PackageDataPut, ProjectData, ProjectDataPut, ProjectID, Projects, Readme, Users, User},
    pagination::PaginationParams,
    version::Version
};

pub async fn not_found() -> Result<(), AppError>
{
    Err(AppError::NotFound)
}

pub async fn root_get() -> &'static str {
    "hello world"
}

pub async fn projects_get(
    Wrapper(Query(params)): Wrapper<Query<PaginationParams>>,
    State(core): State<CoreArc>
) -> Result<Json<Projects>, AppError>
{
    Ok(
        Json(
            core.get_projects(
                params.seek.unwrap_or_default(),
                params.limit.unwrap_or_default()
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
    Wrapper(Json(proj_data)): Wrapper<Json<ProjectDataPut>>
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
    Wrapper(Json(owners)): Wrapper<Json<Users>>
) -> Result<(), AppError>
{
    core.add_owners(&owners, proj_id.0).await
}

pub async fn owners_remove(
    Owned(_, proj_id): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(owners)): Wrapper<Json<Users>>
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

pub async fn packages_put(
    _proj_id: ProjectID,
    State(_core): State<CoreArc>,
    Wrapper(Json(_pkg_data)): Wrapper<Json<PackageDataPut>>
) -> Result<(), AppError>
{
    todo!();
}

pub async fn release_get(
    proj_id: ProjectID,
    pkg_id: PackageID,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_release(proj_id.0, pkg_id.0).await?))
}


// TODO: Version extractor?
pub async fn release_version_get(
    ProjectIDAndPackageID((proj_id, pkg_id)): ProjectIDAndPackageID,
    Path((_, _, version)): Path<(String, String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    let version = version.parse::<Version>()
        .or(Err(AppError::NotFound))?;

    Ok(
        Redirect::to(
            &core.get_release_version(proj_id.0, pkg_id.0, &version).await?
        )
    )
}

pub async fn release_put(
    _proj_id: ProjectID,
    Path((_pkg_name, _version)): Path<(String, String)>,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}

/*
pub async fn release_put(
    _: Owner,
    proj_id: ProjectID,
    pkg_id: PackageID,
    Path((_, _, pkg_version): Path<(String, String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    // 307 preserves the original method and body, which is essential
    // for a PUT uploading a file; we cannot use a 303 here
    Ok(
        Redirect:temporary(

    core.pacakge_version_put(proj_id.0, pkg_id.0, &pkg_version, ).await
}
*/

pub async fn readme_get(
    Path(readme_id): Path<u32>,
    State(core): State<CoreArc>
) -> Result<Json<Readme>, AppError>
{
    Ok(Json(core.get_readme(readme_id).await?))
}

pub async fn image_get(
    proj_id: ProjectID,
    Path(img_name): Path<String>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_image(proj_id.0, img_name).await?))
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
