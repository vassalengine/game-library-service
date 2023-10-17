use crate::{
    errors::AppError,
    model::Users
};

pub trait DataStore: Clone + Send {
    async fn user_is_owner(
        &self,
        user: &str,
        proj_id: u32
    ) -> Result<bool, AppError>;

    async fn add_owners(
        &self,
        owners: &[String],
        proj_id: u32
    ) -> Result<(), AppError>;

    async fn remove_owners(
        &self,
        owners: &[String],
        proj_id: u32
    ) -> Result<(), AppError>;

    async fn get_owners(
        &self,
        proj_id: u32
    ) -> Result<Users, AppError>;
}
