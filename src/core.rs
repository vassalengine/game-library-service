use axum::{
    async_trait,
    body::Bytes
};
use futures::Stream;
use mime::Mime;
use std::{
    io,
    mem,
    sync::Arc
};
use thiserror::Error;

use crate::{
    model::{Owner, PackageDataPost, Package, Projects, ProjectData, ProjectDataPatch, ProjectDataPost, Project, User, Users},
    params::ProjectsParams,
    pagination,
    time,
    version::Version
};

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Unsupported media type")]
    BadMimeType,
    #[error("File too large")]
    TooLarge,
    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,
    #[error("Invalid project name")]
    InvalidProjectName,
    #[error("Project name in use")]
    ProjectNameInUse,
    #[error("Malformed query")]
    MalformedQuery,
    #[error("Not a found")]
    NotFound,
    #[error("Not a package")]
    NotAPackage,
    #[error("Not a project")]
    NotAProject,
    #[error("Not a revision")]
    NotARevision,
    #[error("Not a user")]
    NotAUser,
    #[error("Not a version")]
    NotAVersion,
    #[error("Internal error")]
    InternalError,
    #[error("{0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("{0}")]
    TimeError(#[from] time::Error),
    #[error("{0}")]
    SeekError(#[from] pagination::SeekError)
}

impl PartialEq for CoreError {
    fn eq(&self, other: &Self) -> bool {
        // sqlx::Error is not PartialEq, so we must exclude it
        mem::discriminant(self) == mem::discriminant(other) &&
        !matches!(self, CoreError::DatabaseError(_))
    }
}

#[async_trait]
pub trait Core {
    async fn get_project_id(
         &self,
        _proj: &str
    ) -> Result<Project, CoreError>
    {
        unimplemented!();
    }

    async fn get_package_id(
         &self,
        _proj: Project,
        _pkg: &str
    ) -> Result<Package, CoreError>
    {
        unimplemented!();
    }

    async fn get_user_id(
         &self,
        _username: &str
    ) -> Result<User, CoreError>
    {
        unimplemented!();
    }

    async fn get_owners(
        &self,
        _proj: Project
    ) -> Result<Users, CoreError>
    {
        unimplemented!();
    }

    async fn add_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn remove_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn user_is_owner(
        &self,
        _user: User,
        _proj: Project
    ) -> Result<bool, CoreError>
    {
        unimplemented!();
    }

    async fn get_projects(
        &self,
        _params: ProjectsParams
    ) -> Result<Projects, CoreError>
    {
        unimplemented!();
    }

    async fn get_project(
        &self,
        _proj: Project
    ) -> Result<ProjectData, CoreError>
    {
        unimplemented!();
    }

    async fn create_project(
        &self,
        _user: User,
        _proj: &str,
        _proj_data: &ProjectDataPost
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn update_project(
        &self,
        _owner: Owner,
        _proj: Project,
        _proj_data: &ProjectDataPatch
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn get_project_revision(
        &self,
        _proj: Project,
        _revision: i64
    ) -> Result<ProjectData, CoreError>
    {
        unimplemented!();
    }

    async fn create_package(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: &str,
        _pkg_data: &PackageDataPost
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn get_release(
        &self,
        _proj: Project,
        _pkg: Package
    ) -> Result<String, CoreError>
    {
        unimplemented!();
    }

    async fn get_release_version(
        &self,
        _proj: Project,
        _pkg: Package,
        _version: &Version
    ) -> Result<String, CoreError>
    {
        unimplemented!();
    }

    async fn add_release(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: Package,
        _version: &Version,
        _filename: &str,
        _stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn get_players(
        &self,
        _proj: Project
    ) -> Result<Users, CoreError>
    {
        unimplemented!();
    }

    async fn add_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn remove_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }

    async fn get_image(
        &self,
        _proj: Project,
        _img_name: &str
    ) -> Result<String, CoreError>
    {
        unimplemented!();
    }

    async fn get_image_revision(
        &self,
        _proj: Project,
        _revision: i64,
        _img_name: &str
    ) -> Result<String, CoreError>
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
        _stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>
    ) -> Result<(), CoreError>
    {
        unimplemented!();
    }
}

pub type CoreArc = Arc<dyn Core + Send + Sync>;
