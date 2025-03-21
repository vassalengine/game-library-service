use async_trait::async_trait;
use axum::body::Bytes;
use futures::Stream;
use mime::Mime;
use std::{
    io,
    mem,
    sync::Arc
};
use thiserror::Error;

use crate::{
    db,
    model::{Owner, PackageDataPost, Package, Projects, ProjectData, ProjectDataPatch, ProjectDataPost, Project, Release, User, Users},
    module,
    params::ProjectsParams,
    pagination,
    time,
    upload,
    version::Version
};

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,
    #[error("Invalid project name")]
    InvalidProjectName,
    #[error("Project name in use")]
    ProjectNameInUse,
    #[error("Not a user")]
    NotAUser,
    #[error("{0}")]
    XDatabaseError(#[from] db::DatabaseError)
}

impl PartialEq for CoreError {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

#[derive(Debug, Error)]
pub enum GetIdError {
    #[error("Not found")]
    NotFound,
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

impl PartialEq for GetIdError {
    fn eq(&self, other: &Self) -> bool {
        // sqlx::Error is not PartialEq, so we must exclude it
        mem::discriminant(self) == mem::discriminant(other) &&
        !matches!(self, Self::DatabaseError(_))
    }
}

#[derive(Debug, Error)]
pub enum UserIsOwnerError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

#[derive(Debug, Error)]
pub enum GetOwnersError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

#[derive(Debug, Error)]
pub enum AddOwnersError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

#[derive(Debug, Error)]
pub enum RemoveOwnersError {
    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

impl PartialEq for RemoveOwnersError {
    fn eq(&self, other: &Self) -> bool {
        // sqlx::Error is not PartialEq, so we must exclude it
        mem::discriminant(self) == mem::discriminant(other) &&
        !matches!(self, Self::DatabaseError(_))
    }
}

#[derive(Debug, Error)]
pub enum GetProjectsError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("Malformed query")]
    MalformedQuery,
    #[error("{0}")]
    SeekError(#[from] pagination::SeekError),
    #[error("{0}")]
    TimeError(#[from] time::Error)
}

#[derive(Debug, Error)]
pub enum GetProjectError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("Not found")]
    NotFound,
    #[error("{0}")]
    TimeError(#[from] time::Error)
}

#[derive(Debug, Error)]
pub enum CreateProjectError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("{0}")]
    TimeError(#[from] time::Error)
}

#[derive(Debug, Error)]
pub enum UpdateProjectError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("{0}")]
    TimeError(#[from] time::Error)
}

#[derive(Debug, Error)]
pub enum CreatePackageError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("{0}")]
    TimeError(#[from] time::Error)
}

#[derive(Debug, Error)]
pub enum CreateReleaseError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("{0}")]
    TimeError(#[from] time::Error)
}

#[derive(Debug, Error)]
pub enum AddFileError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("{0}")]
    IOError(#[from] io::Error),
    #[error("{0}")]
    ModuleError(#[from] module::Error),
    #[error("{0}")]
    TimeError(#[from] time::Error),
    #[error("File too large")]
    TooLarge,
    #[error("{0}")]
    UploadError(#[from] upload::UploadError)
}

#[derive(Debug, Error)]
pub enum GetPlayersError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

#[derive(Debug, Error)]
pub enum AddPlayerError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

#[derive(Debug, Error)]
pub enum RemovePlayerError {
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

#[derive(Debug, Error)]
pub enum GetImageError {
    #[error("Not found")]
    NotFound,
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError)
}

impl PartialEq for GetImageError {
    fn eq(&self, other: &Self) -> bool {
        // sqlx::Error is not PartialEq, so we must exclude it
        mem::discriminant(self) == mem::discriminant(other) &&
        !matches!(self, Self::DatabaseError(_))
    }
}

#[derive(Debug, Error)]
pub enum AddImageError {
    #[error("Unsupported media type")]
    BadMimeType,
    #[error("{0}")]
    DatabaseError(#[from] db::DatabaseError),
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("{0}")]
    IOError(#[from] io::Error),
    #[error("{0}")]
    TimeError(#[from] time::Error),
    #[error("File too large")]
    TooLarge,
    #[error("{0}")]
    UploadError(#[from] upload::UploadError)
}

#[async_trait]
pub trait Core {
    fn max_file_size(&self) -> usize {
        unimplemented!();
    }

    fn max_image_size(&self) -> usize {
        unimplemented!();
    }

    async fn get_project_id(
         &self,
        _proj: &str
    ) -> Result<Project, GetIdError>
    {
        unimplemented!();
    }

    async fn get_package_id(
         &self,
        _proj: Project,
        _pkg: &str
    ) -> Result<Package, GetIdError>
    {
        unimplemented!();
    }

    async fn get_project_package_ids(
         &self,
        _proj: &str,
        _pkg: &str
    ) -> Result<(Project, Package), GetIdError>
    {
        unimplemented!();
    }

    async fn get_release_id(
         &self,
        _proj: Project,
        _pkg: Package,
        _release: &str
    ) -> Result<Release, GetIdError>
    {
        unimplemented!();
    }

    async fn get_project_package_release_ids(
        &self,
        _proj: &str,
        _pkg: &str,
        _release: &str
    ) -> Result<(Project, Package, Release), GetIdError>
    {
        unimplemented!();
    }

    async fn get_user_id(
         &self,
        _username: &str
    ) -> Result<User, GetIdError>
    {
        unimplemented!();
    }

    async fn get_owners(
        &self,
        _proj: Project
    ) -> Result<Users, GetOwnersError>
    {
        unimplemented!();
    }

    async fn add_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), AddOwnersError>
    {
        unimplemented!();
    }

    async fn remove_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), RemoveOwnersError>
    {
        unimplemented!();
    }

    async fn user_is_owner(
        &self,
        _user: User,
        _proj: Project
    ) -> Result<bool, UserIsOwnerError>
    {
        unimplemented!();
    }

    async fn get_projects(
        &self,
        _params: ProjectsParams
    ) -> Result<Projects, GetProjectsError>
    {
        unimplemented!();
    }

    async fn get_project(
        &self,
        _proj: Project
    ) -> Result<ProjectData, GetProjectError>
    {
        unimplemented!();
    }

    async fn create_project(
        &self,
        _user: User,
        _proj: &str,
        _proj_data: &ProjectDataPost
    ) -> Result<(), CreateProjectError>
    {
        unimplemented!();
    }

    async fn update_project(
        &self,
        _owner: Owner,
        _proj: Project,
        _proj_data: &ProjectDataPatch
    ) -> Result<(), UpdateProjectError>
    {
        unimplemented!();
    }

    async fn get_project_revision(
        &self,
        _proj: Project,
        _revision: i64
    ) -> Result<ProjectData, GetProjectError>
    {
        unimplemented!();
    }

    async fn create_package(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: &str,
        _pkg_data: &PackageDataPost
    ) -> Result<(), CreatePackageError>
    {
        unimplemented!();
    }

    async fn create_release(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: Package,
        _version: &Version
    ) -> Result<(), CreateReleaseError>
    {
        unimplemented!();
    }

    async fn add_file(
        &self,
        _owner: Owner,
        _proj: Project,
        _release: Release,
        _requires: Option<&str>,
        _filename: &str,
        _content_length: Option<u64>,
        _stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>
    ) -> Result<(), AddFileError>
    {
        unimplemented!();
    }

    async fn get_players(
        &self,
        _proj: Project
    ) -> Result<Users, GetPlayersError>
    {
        unimplemented!();
    }

    async fn add_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), AddPlayerError>
    {
        unimplemented!();
    }

    async fn remove_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), RemovePlayerError>
    {
        unimplemented!();
    }

    async fn get_image(
        &self,
        _proj: Project,
        _img_name: &str
    ) -> Result<String, GetImageError>
    {
        unimplemented!();
    }

    async fn get_image_revision(
        &self,
        _proj: Project,
        _revision: i64,
        _img_name: &str
    ) -> Result<String, GetImageError>
    {
        unimplemented!();
    }

    async fn add_image(
        &self,
        _owner: Owner,
        _proj: Project,
        _img_name: &str,
        _content_type: &Mime,
        _content_length: Option<u64>,
        _stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>
    ) -> Result<(), AddImageError>
    {
        unimplemented!();
    }
}

pub type CoreArc = Arc<dyn Core + Send + Sync>;
