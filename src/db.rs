use axum::async_trait;
use serde::Deserialize;

use crate::{
    errors::AppError,
    model::{GameData, ProjectID, ProjectDataPut, ProjectSummary, Readme, User, Users},
    version::Version
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
    pub game_year: String,
    pub readme_id: i64
}

impl From<ProjectRow> for ProjectSummary {
    fn from(r: ProjectRow) -> Self {
        ProjectSummary {
            name: r.name,
            description: r.description,
            revision: r.revision,
            created_at: r.created_at,
            modified_at: r.modified_at,
            tags: vec![],
            game: GameData {
                title: r.game_title,
                title_sort_key: r.game_title_sort,
                publisher: r.game_publisher,
                year: r.game_year
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ProjectRevisionRow {
    pub revision: i64,
    pub project_data_id: i64,
    pub readme_id: i64,
    pub modified_at: String
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PackageRow {
    pub package_id: i64,
    pub name: String,
    pub created_at: String
//    description: String
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ReleaseRow {
    pub release_id: i64,
    pub version: String,
    pub version_major: i64,
    pub version_minor: i64,
    pub version_patch: i64,
    pub version_pre: String,
    pub version_build: String,
    pub filename: String,
    pub url: String,
    pub size: i64,
    pub checksum: String,
    pub published_at: String,
    pub published_by: String
//    requires: String
}

#[async_trait]
pub trait DatabaseClient {
    async fn get_project_id(
        &self,
        _project: &str
    ) -> Result<ProjectID, AppError>
    {
        unimplemented!();
    }

    async fn get_project_count(
        &self,
    ) -> Result<i32, AppError>
    {
        unimplemented!();
    }

    async fn get_user_id(
        &self,
        _user: &str
    ) -> Result<i64, AppError>
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

    async fn user_is_owner(
        &self,
        _user: &User,
        _proj_id: i64
    ) -> Result<bool, AppError>
    {
        unimplemented!();
    }

    async fn add_owner(
        &self,
        _user_id: i64,
        _proj_id: i64
    ) -> Result<(), AppError>
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

    async fn remove_owner(
        &self,
        _user_id: i64,
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

    async fn has_owner(
        &self,
        _proj_id: i64,
    ) -> Result<bool, AppError>
    {
        unimplemented!();
    }

    async fn get_projects_start_window(
        &self,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        unimplemented!();
    }

    async fn get_projects_end_window(
        &self,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        unimplemented!();
    }

    async fn get_projects_after_window(
        &self,
        _name: &str,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        unimplemented!();
    }

    async fn get_projects_before_window(
        &self,
        _name: &str,
        _limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        unimplemented!();
    }

    async fn create_project(
        &self,
        _user: &User,
        _proj: &str,
        _proj_data: &ProjectDataPut,
        _now: &str
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn copy_project_revision(
        &self,
        _proj_id: i64
    ) -> Result<i64, AppError>
    {
        unimplemented!();
    }

    async fn update_project(
        &self,
        _proj_id: i64,
        _proj_data: &ProjectDataPut,
        _now: &str
    ) -> Result<(), AppError>
    {
        unimplemented!();
    }

    async fn get_project_row(
        &self,
        _proj_id: i64
    ) -> Result<ProjectRow, AppError>
    {
        unimplemented!();
    }

    async fn get_project_row_revision(
        &self,
        _proj_id: i64,
        _revision: u32
    ) -> Result<ProjectRow, AppError>
    {
        unimplemented!();
    }

    async fn get_packages(
        &self,
        _proj_id: i64
    ) -> Result<Vec<PackageRow>, AppError>
    {
        unimplemented!();
    }

    async fn get_packages_at(
        &self,
        _proj_id: i64,
        _date: &str,
    ) -> Result<Vec<PackageRow>, AppError>
    {
        unimplemented!();
    }

    async fn get_releases(
        &self,
        _pkg_id: i64
    ) -> Result<Vec<ReleaseRow>, AppError>
    {
        unimplemented!();
    }

    async fn get_releases_at(
        &self,
        _pkg_id: i64,
        _date: &str
    ) -> Result<Vec<ReleaseRow>, AppError>
    {
        unimplemented!();
    }

    async fn get_authors(
        &self,
        _pkg_ver_id: i64
    ) -> Result<Users, AppError>
    {
        unimplemented!();
    }

    async fn get_package_url(
        &self,
        _pkg_id: i64
    ) -> Result<String, AppError>
    {
        unimplemented!();
    }

    async fn get_release_url(
        &self,
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
        _proj_id: i64,
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

    async fn get_readme(
        &self,
        _readme_id: i64
    ) -> Result<Readme, AppError>
    {
        unimplemented!();
    }

    async fn add_readme(
        &self,
        _text: &str
    ) -> Result<i64, AppError>
    {
        unimplemented!();
    }
}
