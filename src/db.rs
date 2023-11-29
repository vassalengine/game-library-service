use axum::async_trait;
use serde::Deserialize;
use sqlx::{Acquire, Executor};

use crate::{
    errors::AppError,
    model::{ProjectID, ProjectDataPut, ProjectSummary, Readme, User, Users},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ProjectRow {
    pub name: String,
    pub description: String,
    pub revision: i64,
    pub created_at: String,
    pub modified_at: String,
    pub game_title: String,
    pub game_title_sort: String,
    pub game_publisher: String,
    pub game_year: String
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PackageRow {
    pub id: i64,
    pub name: String,
//    description: String
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct VersionRow {
    pub version: String,
    pub filename: String,
    pub url: String,
/*
    size: u64,
    checksum: String,
    published_at: String,
    published_by: String,
    requires: String
*/
}

#[async_trait]
pub trait DatabaseOperations<DB> {
    async fn get_project_id<'e, E>(
        &self,
        _ex: E,
        _project: &str
    ) -> Result<ProjectID, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_project_count<'e, E>(
        &self,
        _ex: E
    ) -> Result<i32, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_user_id<'e, E>(
        &self,
        _ex: E,
        _user: &str
    ) -> Result<i64, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_owners<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64
    ) -> Result<Users, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn user_is_owner<'e, E>(
        &self,
        _ex: E,
        _user: &User,
        _proj_id: i64
    ) -> Result<bool, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn add_owner<'e, E>(
        &self,
        _ex: E,
        _user_id: i64,
        _proj_id: i64
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn remove_owner<'e, E>(
        &self,
        _ex: E,
        _user_id: i64,
        _proj_id: i64
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn has_owner<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64,
    ) -> Result<bool, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_projects_start_window<'e, E>(
        &self,
        _ex: E,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_projects_end_window<'e, E>(
        &self,
        _ex: E,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_projects_after_window<'e, E>(
        &self,
        _ex: E,
        _name: &str,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_projects_before_window<'e, E>(
        &self,
        _ex: E,
        _name: &str,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn create_project<'e, E>(
        &self,
        _ex: E,
        _proj: &str,
        _proj_data: &ProjectDataPut,
        _game_title_sort_key: &str,
        _now: &str
    ) -> Result<i64, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn copy_project_revision<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64
    ) -> Result<i64, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn update_project<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64,
        _revision: i64,
        _proj_data: &ProjectDataPut,
        _now: &str
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_project_row<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64
    ) -> Result<ProjectRow, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_project_row_revision<'a, A>(
        &self,
        _conn: A,
        _proj_id: i64,
        _revision: u32
    ) -> Result<ProjectRow, AppError>
    where
        A: Acquire<'a, Database = DB> + Send
    {
        unimplemented!();
    }

    async fn get_packages<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64
    ) -> Result<Vec<PackageRow>, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_versions<'e, E>(
        &self,
        _ex: E,
        _pkg_id: i64
    ) -> Result<Vec<VersionRow>, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    } 

    async fn get_package_url<'e, E>(
        &self,
        _ex: E,
        _pkg_id: i64
    ) -> Result<String, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_players<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64
    ) -> Result<Users, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn add_player<'e, E>(
        &self,
        _ex: E,
        _user_id: i64,
        _proj_id: i64,
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn remove_player<'e, E>(
        &self,
        _ex: E,
        _user_id: i64,
        _proj_id: i64
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_readme<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64
    ) -> Result<Readme, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }

    async fn get_readme_revision<'e, E>(
        &self,
        _ex: E,
        _proj_id: i64,
        _revision: u32
    ) -> Result<Readme, AppError>
    where
        E: Executor<'e, Database = DB>
    {
        unimplemented!();
    }
}
