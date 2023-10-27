use axum::async_trait;
use std::sync::Arc;

use crate::{
    errors::AppError,
    model::{Project, ProjectID, User, UserID, Users}
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
}

pub type CoreArc = Arc<dyn Core + Send + Sync>;
