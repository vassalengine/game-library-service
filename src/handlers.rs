use axum::{
    extract::{Path, Query, Request, State},
    response::{Json, Redirect}
};
use axum_extra::{
    TypedHeader,
    headers::{ContentLength, ContentType}
};
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use std::io;

use crate::{
    core::CoreArc,
    errors::AppError,
    extractors::{ProjectPackage, ProjectPackageVersion, Wrapper},
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
    Ok(core.create_project(owner, &proj, &proj_data).await?)
}

pub async fn project_patch(
    Owned(owner, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(proj_data)): Wrapper<Json<ProjectDataPatch>>
) -> Result<(), AppError>
{
    Ok(core.update_project(owner, proj, &proj_data).await?)
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
    Ok(core.add_owners(&owners, proj).await?)
}

pub async fn owners_remove(
    Owned(_, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(owners)): Wrapper<Json<Users>>
) -> Result<(), AppError>
{
    Ok(core.remove_owners(&owners, proj).await?)
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
    Ok(core.add_player(requester, proj).await?)
}

pub async fn players_remove(
    requester: User,
    proj: Project,
    State(core): State<CoreArc>,
) -> Result<(), AppError>
{
    Ok(core.remove_player(requester, proj).await?)
}

pub async fn packages_post(
    Owned(owner, proj): Owned,
    Path((_, pkg)): Path<(String, String)>,
    State(core): State<CoreArc>,
    Wrapper(Json(pkg_data)): Wrapper<Json<PackageDataPost>>
) -> Result<(), AppError>
{
    Ok(core.create_package(owner, proj, &pkg, &pkg_data).await?)
}

// TODO
//pub async fn packages_patch(

pub async fn release_get(
    ProjectPackage(proj, pkg): ProjectPackage,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_release(proj, pkg).await?))
}

pub async fn release_version_get(
    ProjectPackageVersion(proj, pkg, version): ProjectPackageVersion,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
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
    let version = version.parse::<Version>()
        .or(Err(AppError::NotAVersion))?;

/*
    let stream = request.into_body()
        .into_data_stream()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err));

    core.add_image(&owner, proj, &img_name, Box::new(stream))
        .await
        .or(Err(AppError::InternalError))?;
*/

    Ok(())
}

pub async fn image_get(
    proj: Project,
    Path((_, img_name)): Path<(String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_image(proj, &img_name).await?))
}

pub async fn image_revision_get(
    proj: Project,
    Path((_, img_name, revision)): Path<(String, String, u32)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(
        Redirect::to(
            &core.get_image_revision(proj, revision as i64, &img_name).await?
        )
    )
}

pub async fn image_post(
    Owned(owner, proj): Owned,
    Path((_, img_name)): Path<(String, String)>,
    content_type: Option<TypedHeader<ContentType>>,
    content_length: Option<TypedHeader<ContentLength>>,
    State(core): State<CoreArc>,
    request: Request
) -> Result<(), AppError>
{
    Ok(
        core.add_image(
            owner,
            proj,
            &img_name,
            &content_type.ok_or(AppError::BadMimeType)?.0.into(),
            content_length.map(|h| h.0.0),
            into_stream(request)
        ).await?
    )
}

pub async fn flag_post(
    _requester: User,
    _proj: Project,
    State(_core): State<CoreArc>
) -> Result<(), AppError>
{
    todo!();
}
