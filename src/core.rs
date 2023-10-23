use axum::async_trait;
use std::sync::Arc;

use crate::{
    errors::AppError,
    model::{User, Users}
};

#[async_trait]
pub trait Core {
    async fn get_owners(
        &self,
        _proj_id: u32
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_owners(
        &self,
        _owners: &Users,
        _proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_owners(
        &self,
        _owners: &Users,
        _proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn user_is_owner(
        &self,
        _user: &User,
        _proj_id: u32
    ) -> Result<bool, AppError>
    {
        unimplemented!();
    }

    async fn get_players(
        &self,
        _proj_id: u32
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_player(
        &self,
        _player: &User,
        _proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_player(
        &self,
        _player: &User,
        _proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }
}

pub type CoreArc = Arc<dyn Core + Send + Sync>;
