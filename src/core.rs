use axum::async_trait;
use std::sync::Arc;

use crate::{
    errors::AppError,
    model::{Owner, PackageID, Project, Projects, ProjectData, ProjectDataPatch, ProjectDataPost, ProjectID, User, UserID, Users},
    pagination::{Limit, Seek},
    version::Version
};

#[async_trait]
pub trait Core {
    async fn get_project_id(
         &self,
        _proj: &Project
    ) -> Result<ProjectID, AppError>
    {
        unimplemented!();
    }

    async fn get_package_id(
         &self,
        _proj_id: i64,
        _pkg: &str
    ) -> Result<PackageID, AppError>
    {
        unimplemented!();
    }

    async fn get_user_id(
         &self,
        _user: &User
    ) -> Result<UserID, AppError>
    {
        unimplemented!();
    }

    async fn get_owners(
        &self,
        _proj_id: i64
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_owners(
        &self,
        _owners: &Users,
        _proj_id: i64
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_owners(
        &self,
        _owners: &Users,
        _proj_id: i64
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn user_is_owner(
        &self,
        _user: &User,
        _proj_id: i64
    ) -> Result<bool, AppError>
    {
        unimplemented!();
    }

    async fn get_projects(
        &self,
        _from: Seek,
        _limit: Limit
    ) -> Result<Projects, AppError>
    {
        unimplemented!();
    }

    async fn get_project(
        &self,
        _proj_id: i64
    ) -> Result<ProjectData, AppError>
    {
        unimplemented!();
    }

    async fn create_project(
        &self,
        _user: &User,
        _proj: &str,
        _proj_data: &ProjectDataPost
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn update_project(
        &self,
        _user: &Owner,
        _proj_id: i64,
        _proj_data: &ProjectDataPatch
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_project_revision(
        &self,
        _proj_id: i64,
        _revision: i64
    ) -> Result<ProjectData, AppError>
    {
        unimplemented!();
    }

    async fn get_release(
        &self,
        _proj_id: i64,
        _pkg_id: i64
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }

    async fn get_release_version(
        &self,
        _proj_id: i64,
        _pkg_id: i64,
        _version: &Version
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }

    async fn get_players(
        &self,
        _proj_id: i64
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_player(
        &self,
        _player: &User,
        _proj_id: i64
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_player(
        &self,
        _player: &User,
        _proj_id: i64
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_image(
        &self,
        _proj_id: i64,
        _img_name: &str
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }
}

pub type CoreArc = Arc<dyn Core + Send + Sync>;
