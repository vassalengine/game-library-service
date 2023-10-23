use axum::async_trait;

use sqlx::sqlite::SqlitePool;

use crate::{
    core::Core,
    errors::AppError,
    model::{User, Users}
};

#[derive(Clone)]
pub struct ProdCore {
    pub db: SqlitePool
}

#[async_trait]
impl Core for ProdCore {
    async fn get_owners(
        &self,
        proj_id: u32
    ) -> Result<Users, AppError>
    {
        todo!();
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        todo!();
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        todo!();
    }

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: u32
    ) -> Result<bool, AppError>
    {
        todo!();
    }
}
