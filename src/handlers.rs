use axum::{
    extract::{Path, Query, State},
    response::{Json, Redirect}
};

use crate::{
    core::CoreArc,
    errors::AppError,
    extractors::{ProjectAndPackage, Wrapper},
    model::{Owned, Package, PackageDataPost, ProjectData, ProjectDataPatch, ProjectDataPost, Project, Projects, Users, User},
    params::ProjectsParams,
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
    Wrapper(Query(params)): Wrapper<Query<ProjectsParams>>,
    State(core): State<CoreArc>
) -> Result<Json<Projects>, AppError>
{
    Ok(Json(core.get_projects(params).await?))
}

pub async fn project_get(
    proj: Project,
    State(core): State<CoreArc>
) -> Result<Json<ProjectData>, AppError>
{
    Ok(Json(core.get_project(proj).await?))
}

pub async fn project_post(
    owner: User,
    Path(proj): Path<String>,
    State(core): State<CoreArc>,
    Wrapper(Json(proj_data)): Wrapper<Json<ProjectDataPost>>
) -> Result<(), AppError>
{
    core.create_project(owner, &proj, &proj_data).await
}

pub async fn project_patch(
    Owned(owner, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(proj_data)): Wrapper<Json<ProjectDataPatch>>
) -> Result<(), AppError>
{
    core.update_project(owner, proj, &proj_data).await
}

pub async fn project_revision_get(
    proj: Project,
    Path((_, revision)): Path<(String, u32)>,
    State(core): State<CoreArc>
) -> Result<Json<ProjectData>, AppError>
{
    Ok(Json(core.get_project_revision(proj, revision as i64).await?))
}

pub async fn owners_get(
    proj: Project,
    State(core): State<CoreArc>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_owners(proj).await?))
}

pub async fn owners_add(
    Owned(_, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(owners)): Wrapper<Json<Users>>
) -> Result<(), AppError>
{
    core.add_owners(&owners, proj).await
}

pub async fn owners_remove(
    Owned(_, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(owners)): Wrapper<Json<Users>>
) -> Result<(), AppError>
{
    core.remove_owners(&owners, proj).await
}

pub async fn players_get(
    proj: Project,
    State(core): State<CoreArc>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_players(proj).await?))
}

pub async fn players_add(
    requester: User,
    proj: Project,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    core.add_player(requester, proj).await
}

pub async fn players_remove(
    requester: User,
    proj: Project,
    State(core): State<CoreArc>,
) -> Result<(), AppError>
{
    core.remove_player(requester, proj).await
}

pub async fn packages_post(
    Owned(owner, proj): Owned,
    Path((_, pkg)): Path<(String, String)>,
    State(core): State<CoreArc>,
    Wrapper(Json(pkg_data)): Wrapper<Json<PackageDataPost>>
) -> Result<(), AppError>
{
    core.create_package(owner, proj, &pkg, &pkg_data).await
}

// TODO
//pub async fn packages_patch(

pub async fn release_get(
    proj: Project,
    pkg: Package,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_release(proj, pkg).await?))
}


// TODO: Version extractor?
pub async fn release_version_get(
    ProjectAndPackage((proj, pkg)): ProjectAndPackage,
    Path((_, _, version)): Path<(String, String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    let version = version.parse::<Version>()
        .or(Err(AppError::NotFound))?;

    Ok(
        Redirect::to(
            &core.get_release_version(proj, pkg, &version).await?
        )
    )
}

pub async fn release_put(
    Owned(owner, proj): Owned,
    Path((_, pkg, version)): Path<(String, String, String)>,
    State(core): State<CoreArc>,
    request: Request
) -> Result<(), AppError>
{
    todo!();
}

/*
pub async fn release_put(
    _: Owner,
    proj: Project,
    pkg: Package,
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

pub async fn image_get(
    proj: Project,
    Path((_, img_name)): Path<(String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_image(proj, &img_name).await?))
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
    _proj: Project,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}
