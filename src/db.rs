use axum::async_trait;
use serde::Deserialize;
use sqlx::FromRow;

use crate::{
    core::CoreError,
    model::{GalleryImage, Owner, Package, PackageDataPost, Project, ProjectDataPatch, ProjectDataPost, Release, User, Users},
    pagination::{Direction, SortBy},
    version::Version
};

#[derive(Debug, Deserialize, FromRow, PartialEq)]
pub struct ProjectSummaryRow {
    pub rank: f64,
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub revision: i64,
    pub created_at: i64,
    pub modified_at: i64,
    pub game_title: String,
    pub game_title_sort: String,
    pub game_publisher: String,
    pub game_year: String,
    pub game_players_min: Option<i64>,
    pub game_players_max: Option<i64>,
    pub game_length_min: Option<i64>,
    pub game_length_max: Option<i64>,
    pub image: Option<String>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ProjectRow {
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub revision: i64,
    pub created_at: i64,
    pub modified_at: i64,
    pub modified_by: i64,
    pub game_title: String,
    pub game_title_sort: String,
    pub game_publisher: String,
    pub game_year: String,
    pub game_players_min: Option<i64>,
    pub game_players_max: Option<i64>,
    pub game_length_min: Option<i64>,
    pub game_length_max: Option<i64>,
    pub image: Option<String>,
    pub readme: String
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct PackageRow {
    pub package_id: i64,
    pub name: String,
    pub created_at: i64
//    description: String
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ReleaseRow {
    pub release_id: i64,
    pub version: String,
    pub version_major: i64,
    pub version_minor: i64,
    pub version_patch: i64,
    pub version_pre: String,
    pub version_build: String,
    pub published_at: i64,
    pub published_by: String
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct FileRow {
    pub id: i64,
    pub filename: String,
    pub url: String,
    pub size: i64,
    pub sha256: String,
    pub requires: String,
    pub published_at: i64,
    pub published_by: String
}

#[async_trait]
pub trait DatabaseClient {
    async fn get_project_id(
        &self,
        _projname: &str
    ) -> Result<Project, CoreError>;

    async fn get_projects_count(
        &self,
    ) -> Result<i64, CoreError>;

    async fn get_projects_query_count(
        &self,
        _query: &str
    ) -> Result<i64, CoreError>;

    async fn get_user_id(
        &self,
        _username: &str
    ) -> Result<User, CoreError>;

    async fn get_owners(
        &self,
        _proj: Project
    ) -> Result<Users, CoreError>;

    async fn user_is_owner(
        &self,
        _user: User,
        _proj: Project
    ) -> Result<bool, CoreError>;

    async fn add_owner(
        &self,
        _user: User,
        _proj: Project
    ) -> Result<(), CoreError>;

    async fn add_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), CoreError>;

    async fn remove_owner(
        &self,
        _user: User,
        _proj: Project
    ) -> Result<(), CoreError>;

    async fn remove_owners(
        &self,
        _owners: &Users,
        _proj: Project
    ) -> Result<(), CoreError>;

    async fn has_owner(
        &self,
        _proj: Project
    ) -> Result<bool, CoreError>;

    async fn get_projects_end_window(
        &self,
        _sort_by: SortBy,
        _dir: Direction,
        _limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>;

    async fn get_projects_query_end_window(
        &self,
        _query: &str,
        _sort_by: SortBy,
        _dir: Direction,
        _limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>;

    async fn get_projects_mid_window(
        &self,
        _sort_by: SortBy,
        _dir: Direction,
        _field: &str,
        _id: u32,
        _limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>;

    async fn get_projects_query_mid_window(
        &self,
        _query: &str,
        _sort_by: SortBy,
        _dir: Direction,
        _field: &str,
        _id: u32,
        _limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>;

    async fn create_project(
        &self,
        _user: User,
        _proj: &str,
        _proj_data: &ProjectDataPost,
        _now: i64
    ) -> Result<(), CoreError>;

    async fn update_project(
        &self,
        _owner: Owner,
        _proj: Project,
        _proj_data: &ProjectDataPatch,
        _now: i64
    ) -> Result<(), CoreError>;

    async fn get_project_row(
        &self,
        proj: Project
    ) -> Result<ProjectRow, CoreError>;

    async fn get_project_row_revision(
        &self,
        _proj: Project,
        _revision: i64
    ) -> Result<ProjectRow, CoreError>;

    async fn get_packages(
        &self,
        _proj: Project
    ) -> Result<Vec<PackageRow>, CoreError>;

    async fn get_packages_at(
        &self,
        _proj: Project,
        _date: i64,
    ) -> Result<Vec<PackageRow>, CoreError>;

    async fn create_package(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: &str,
        _pkg_data: &PackageDataPost,
        _now: i64
    ) -> Result<(), CoreError>;

    async fn get_releases(
        &self,
        _pkg: Package
    ) -> Result<Vec<ReleaseRow>, CoreError>;

    async fn get_releases_at(
        &self,
        _pkg: Package,
        _date: i64
    ) -> Result<Vec<ReleaseRow>, CoreError>;

    async fn get_files(
        &self,
        _rel: Release
    ) -> Result<Vec<FileRow>, CoreError>;

    async fn get_files_at(
        &self,
        _rel: Release,
        _date: i64
    ) -> Result<Vec<FileRow>, CoreError>;

    async fn get_authors(
        &self,
        _pkg_ver_id: i64
    ) -> Result<Users, CoreError>;

    async fn get_release_url(
        &self,
        _pkg: Package
    ) -> Result<String, CoreError>;

    async fn get_release_version_url(
        &self,
        _pkg: Package,
        _version: &Version
    ) -> Result<String, CoreError>;

    async fn add_release_url(
        &self,
        _owner: Owner,
        _proj: Project,
        _pkg: Package,
        _version: &Version,
        _filename: &str,
        _size: i64,
        _sha256: &str,
        _requires: &str,
        _url: &str,
        _now: i64
    ) -> Result<(), CoreError>;

    async fn get_players(
        &self,
        _proj: Project
    ) -> Result<Users, CoreError>;

    async fn add_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), CoreError>;

    async fn remove_player(
        &self,
        _player: User,
        _proj: Project
    ) -> Result<(), CoreError>;

    async fn get_image_url(
        &self,
        _proj: Project,
        _img_name: &str
    ) -> Result<String, CoreError>;

    async fn get_image_url_at(
        &self,
        _proj: Project,
        _img_name: &str,
        _date: i64
    ) -> Result<String, CoreError>;

    async fn add_image_url(
        &self,
        _owner: Owner,
        _proj: Project,
        _img_name: &str,
        _url: &str,
        _now: i64
    ) -> Result<(), CoreError>;

    async fn get_tags(
        &self,
        _proj: Project
    ) -> Result<Vec<String>, CoreError>;

    async fn get_tags_at(
        &self,
        _proj: Project,
        _date: i64
    ) -> Result<Vec<String>, CoreError>;

    async fn get_gallery(
        &self,
        _proj: Project
    ) -> Result<Vec<GalleryImage>, CoreError>;

    async fn get_gallery_at(
        &self,
        _proj: Project,
        _date: i64
    ) -> Result<Vec<GalleryImage>, CoreError>;
}
