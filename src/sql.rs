use axum::async_trait;
use serde::Deserialize;
use sqlx::{Acquire, Executor};

use crate::{
    errors::AppError,
    model::{GameData, ProjectID, ProjectDataPut, ProjectSummary, Readme, User, Users},
};

//type Database = sqlx::sqlite::Sqlite;
//pub type Pool = sqlx::Pool<sqlx::sqlite::Sqlite>;

/*
#[async_trait]
pub trait DatabaseOperations {
    async fn get_project_id(
        self,
        _project: &str
    ) -> Result<ProjectID, AppError>
    {
        unimplemented!();
    }
}

impl<'a> DatabaseOperations for &'a sqlx::SqlitePool {
    async fn get_project_id(
        self,
        project: &str
    ) -> Result<ProjectID, AppError>
    {
        get_project_id(self, project).await
    }
}

impl<'a> DatabaseOperations for &'a mut sqlx::Transaction<'static, sqlx::sqlite::Sqlite> {
    async fn get_project_id(
        self,
        project: &str
    ) -> Result<ProjectID, AppError>
    {
        get_project_id(&mut *self, project).await
    }
}
*/

/*
#[async_trait]
pub trait DatabaseTransaction: DatabaseOperations {
    async fn commit(self) -> Result<(), AppError>; 

    async fn rollback(self) -> Result<(), AppError>;
}

#[async_trait]
pub trait DatabaseClient: DatabaseOperations {
    async fn begin(&self) -> Result<Box<dyn DatabaseTransaction>, AppError>;
}

#[derive(Clone)]
pub struct SqliteClient(pub sqlx::SqlitePool);

pub struct SqliteTransaction(pub sqlx::Transaction<'static, sqlx::sqlite::Sqlite>);

#[async_trait]
impl DatabaseClient for SqliteClient {
    async fn begin(&self) -> Result<Box<dyn DatabaseTransaction>, AppError> {
        Ok(Box::new(SqliteTransaction(self.0.begin().await?)))
    }
}

#[async_trait]
impl<'a> DatabaseOperations for &'a SqliteClient {
    async fn get_project_id(
        &self,
        project: &str
    ) -> Result<ProjectID, AppError>
    {
        get_project_id(&self.0, project).await 
    }
}

#[async_trait]
impl DatabaseTransaction for SqliteTransaction {
    async fn commit(self) -> Result<(), AppError> {
        Ok(self.0.commit().await?)
    }

    async fn rollback(self) -> Result<(), AppError> {
        Ok(self.0.rollback().await?)
    }
}
*/

