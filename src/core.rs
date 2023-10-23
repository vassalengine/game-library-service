use axum::async_trait;

use crate::{
    errors::AppError,
    model::{User, Users}
};

#[async_trait]
pub trait Core {
    async fn get_owners(
        &self,
        proj_id: u32
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: u32
    ) -> Result<bool, AppError>
    {
        unimplemented!();
    }

    async fn get_players(
        &self,
        proj_id: u32
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn add_player(
        &self,
        player: &User,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn remove_player(
        &self,
        player: &User,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }
}
