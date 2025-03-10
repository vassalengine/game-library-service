use axum::{
    body::Bytes,
    extract::{Path, Query, Request, State},
    response::{Json, Redirect}
};
use axum_extra::{
    TypedHeader,
    headers::{ContentLength, ContentType}
};
use futures::{Stream, TryStreamExt};
use http_body_util::{BodyExt, Limited, LengthLimitError};
use std::{
    error::Error,
    io
};

use crate::{
    core::CoreArc,
    errors::AppError,
    extractors::{ProjectPackage, ProjectPackageRelease, ProjectPackageVersion, Wrapper},
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

pub async fn release_post(
    Owned(owner, proj): Owned,
    ProjectPackageVersion(_, pkg, version): ProjectPackageVersion,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    Ok(core.create_release(owner, proj, pkg, &version).await?)
}

fn unpack_limited_error(e: Box<dyn Error + Sync + Send>) -> io::Error {
    // turn boxed error back into io::Error
    match e.downcast::<io::Error>() {
        Ok(e) => *e,
        Err(e) => match e.downcast::<LengthLimitError>() {
            Ok(e) => io::Error::new(io::ErrorKind::FileTooLarge, e),
            Err(e) => io::Error::new(io::ErrorKind::Other, e)
        }
    }
}

fn into_stream(
    request: Request,
    limit: usize
) -> Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>
{
     Box::new(
        Limited::new(request.into_body(), limit)
            .into_data_stream()
            .map_err(unpack_limited_error)
    )
}

fn limit_content_length(
    content_length: Option<u64>,
    max_size: usize
) -> Result<(Option<u64>, usize), AppError>
{
    content_length
        .map_or(
            Some((None, max_size)),
            |cl| cl.try_into().map(|cl| (Some(cl as u64), cl)).ok()
        )
        .filter(|(_, lim)| *lim <= max_size)
        .ok_or(AppError::TooLarge)
}

pub async fn file_post(
    Owned(owner, proj): Owned,
    ProjectPackageRelease(_, pkg, release): ProjectPackageRelease,
    Path((_, _, _, filename)): Path<(String, String, String, String)>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    content_length: Option<TypedHeader<ContentLength>>,
    State(core): State<CoreArc>,
    request: Request
) -> Result<(), AppError>
{
    let (content_length, limit) = limit_content_length(
        content_length.map(|cl| cl.0.0),
        core.max_file_size()
    )?;

    Ok(
        core.add_file(
            owner,
            proj,
            release,
// TODO: where to get requires? read from vmod?
            None,
            &filename,
            content_length,
            into_stream(request, limit)
        ).await?
    )
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
    let (content_length, limit) = limit_content_length(
        content_length.map(|cl| cl.0.0),
        core.max_file_size()
    )?;

    // NB: No ContentType header will result in BAD_REQUEST by default, so
    // have to make it optional and check manually
    Ok(
        core.add_image(
            owner,
            proj,
            &img_name,
            &content_type.ok_or(AppError::BadMimeType)?.0.into(),
            content_length,
            into_stream(request, limit)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn limit_content_length_under_limit() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some(len as u64), len + 1).unwrap(),
            (Some(len as u64), len)
        );
    }

    #[test]
    fn limit_content_length_at_limit() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some(len as u64), len).unwrap(),
            (Some(len as u64), len)
        );
    }

    #[test]
    fn limit_content_length_too_long() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some((len as u64) + 1), len).unwrap_err(),
            AppError::TooLarge
        );
    }

    #[test]
    fn limit_content_length_way_too_long() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some(u64::MAX), len).unwrap_err(),
            AppError::TooLarge
        );
    }

    #[test]
    fn limit_content_length_no_content_length() {
        let len = 20;
        assert_eq!(
            limit_content_length(None, len).unwrap(),
            (None, len)
        );
    }
}
