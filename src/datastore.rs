use axum::async_trait;

use crate::{
    model::{User, Users}
};

#[derive(Debug)]
pub enum DataStoreError {
    Problem(String)
}

#[async_trait]
pub trait DataStore {
    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: u32
    ) -> Result<bool, DataStoreError>;

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), DataStoreError>;

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), DataStoreError>;

    async fn get_owners(
        &self,
        proj_id: u32
    ) -> Result<Users, DataStoreError>;
}
