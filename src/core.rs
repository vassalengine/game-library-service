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
    ) -> Result<Users, AppError>;

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>;

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>;

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: u32
    ) -> Result<bool, AppError>;
}
