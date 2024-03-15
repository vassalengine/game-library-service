use axum::{
    async_trait,
    body::Bytes
};
use futures::Stream;
use std::{
    io,
    sync::Arc
};

use crate::{
    errors::AppError,
    model::{Owner, PackageDataPost, Package, Projects, ProjectData, ProjectDataPatch, ProjectDataPost, Project, User, Users},
    params::ProjectsParams,
    version::Version
};

#[async_trait]
pub trait Core {
    async fn get_project_id(
         &self,
        _proj: &str
    ) -> Result<Project, AppError>
    {
        unimplemented!();
    }

    async fn get_package_id(
         &self,
        _proj: Project,
        _pkg: &str
    ) -> Result<Package, AppError>
    {
        unimplemented!();
    }

    async fn get_user_id(
         &self,
        _username: &str
    ) -> Result<User, AppError>
    {
        unimplemented!();
    }

    async fn get_owners(
        &self,
        _proj: Project
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn user_is_owner(
        &self,
        _user: User,
        _proj: Project
    ) -> Result<bool, AppError>
    {
        unimplemented!();
    }

    async fn get_projects(
        &self,
        _params: ProjectsParams
    ) -> Result<Projects, AppError>
    {
        unimplemented!();
    }

    async fn get_project(
        &self,
        _proj: Project
    ) -> Result<ProjectData, AppError>
    {
        unimplemented!();
    }

    async fn create_project(
        &self,
        _user: User,
        _proj: &str,
        _proj_data: &ProjectDataPost
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn update_project(
        &self,
        _owner: Owner,
        _proj: Project,
        _proj_data: &ProjectDataPatch
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_project_revision(
        &self,
        _proj: Project,
        _revision: i64
    ) -> Result<ProjectData, AppError>
    {
        unimplemented!();
    }

    async fn create_package(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: &str,
        _pkg_data: &PackageDataPost
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_release(
        &self,
        _proj: Project,
        _pkg: Package
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }

    async fn get_release_version(
        &self,
        _proj: Project,
        _pkg: Package,
        _version: &Version
    ) -> Result<String, AppError>
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
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_players(
        &self,
        _proj: Project
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_image(
        &self,
        _proj: Project,
        _img_name: &str
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }

    async fn get_image_revision(
        &self,
        _proj: Project,
        _revision: i64,
        _img_name: &str
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }

    async fn add_image(
        &self,
        _owner: Owner,
        _proj: Project,
        _img_name: &str,
        _stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }
}

pub type CoreArc = Arc<dyn Core + Send + Sync>;
