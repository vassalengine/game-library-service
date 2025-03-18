use sqlx::{
    Database, Executor,
    sqlite::Sqlite
};

mod images;
mod packages;
mod players;
mod project;
mod projects;
mod releases;
mod tags;
mod users;

use crate::{
    core::CoreError,
    db::{DatabaseClient, DatabaseError, FileRow, PackageRow, ProjectRow, ProjectSummaryRow, ReleaseRow},
    model::{GalleryImage, Owner, Package, PackageDataPost, Project, ProjectDataPatch, ProjectDataPost, Release, User, Users},
    pagination::{Direction, SortBy},
    time::rfc3339_to_nanos,
    version::Version
};

pub type Pool = sqlx::Pool<Sqlite>;

#[derive(Clone)]
pub struct SqlxDatabaseClient<DB: Database>(pub sqlx::Pool<DB>);

impl DatabaseClient for SqlxDatabaseClient<Sqlite> {
    async fn get_project_id(
        &self,
        projname: &str
    ) -> Result<Option<Project>, DatabaseError>
    {
        project::get_project_id(&self.0, projname).await
    }

    async fn get_projects_count(
        &self,
    ) -> Result<i64, DatabaseError>
    {
        projects::get_projects_count(&self.0).await
    }

    async fn get_projects_query_count(
        &self,
        query: &str
    ) -> Result<i64, DatabaseError>
    {
        projects::get_projects_query_count(&self.0, query).await
    }

    async fn get_user_id(
        &self,
        username: &str
    ) -> Result<User, CoreError>
    {
        users::get_user_id(&self.0, username).await
    }

    async fn get_owners(
        &self,
        proj: Project
    ) -> Result<Users, CoreError>
    {
        users::get_owners(&self.0, proj).await
    }

    async fn user_is_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<bool, DatabaseError>
    {
        users::user_is_owner(&self.0, user, proj).await
    }

    async fn add_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<(), DatabaseError>
    {
        users::add_owner(&self.0, user, proj).await
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), CoreError>
    {
        users::add_owners(&self.0, owners, proj).await
    }

    async fn remove_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<(), DatabaseError>
    {
        users::remove_owner(&self.0, user, proj).await
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), CoreError>
    {
        users::remove_owners(&self.0, owners, proj).await
    }

    async fn has_owner(
        &self,
        proj: Project
    ) -> Result<bool, DatabaseError>
    {
        users::has_owner(&self.0, proj).await
    }

    async fn get_projects_end_window(
        &self,
        sort_by: SortBy,
        dir: Direction,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>
    {
        projects::get_projects_end_window(&self.0, sort_by, dir, limit).await
    }

    async fn get_projects_query_end_window(
        &self,
        query: &str,
        sort_by: SortBy,
        dir: Direction,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>
    {
        projects::get_projects_query_end_window(&self.0, query, sort_by, dir, limit).await
    }

    async fn get_projects_mid_window(
        &self,
        sort_by: SortBy,
        dir: Direction,
        field: &str,
        id: u32,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>
    {
        match sort_by {
            SortBy::CreationTime |
            SortBy::ModificationTime => projects::get_projects_mid_window(
                &self.0,
                sort_by,
                dir,
                &rfc3339_to_nanos(field)?,
                id,
                limit
            ).await,
            _ => projects::get_projects_mid_window(
                &self.0,
                sort_by,
                dir,
                &field,
                id,
                limit
            ).await
        }
    }

    async fn get_projects_query_mid_window(
        &self,
        query: &str,
        sort_by: SortBy,
        dir: Direction,
        field: &str,
        id: u32,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>
    {
        match sort_by {
            SortBy::CreationTime |
            SortBy::ModificationTime => projects::get_projects_query_mid_window(
                &self.0,
                query,
                sort_by,
                dir,
                &rfc3339_to_nanos(field)?,
                id,
                limit
            ).await,
            SortBy::Relevance => projects::get_projects_query_mid_window(
                &self.0,
                query,
                sort_by,
                dir,
                &field.parse::<f64>().map_err(|_| CoreError::MalformedQuery)?,
                id,
                limit
            ).await,
            _ => projects::get_projects_query_mid_window(
                &self.0,
                query,
                sort_by,
                dir,
                &field,
                id,
                limit
            ).await
        }
    }

    async fn create_project(
        &self,
        user: User,
        proj: &str,
        proj_data: &ProjectDataPost,
        now: i64
    ) -> Result<(), DatabaseError>
    {
        project::create_project(&self.0, user, proj, proj_data, now).await
    }

    async fn update_project(
        &self,
        owner: Owner,
        proj: Project,
        proj_data: &ProjectDataPatch,
        now: i64
    ) -> Result<(), DatabaseError>
    {
        project::update_project(&self.0, owner, proj, proj_data, now).await
    }

    async fn get_project_row(
        &self,
        proj: Project
    ) -> Result<ProjectRow, DatabaseError>
    {
        project::get_project_row(&self.0, proj).await
    }

    async fn get_project_row_revision(
        &self,
        proj: Project,
        revision: i64
    ) -> Result<ProjectRow, DatabaseError>
    {
        project::get_project_row_revision(&self.0, proj, revision).await
    }

    async fn get_packages(
        &self,
        proj: Project
    ) -> Result<Vec<PackageRow>, DatabaseError>
    {
        packages::get_packages(&self.0, proj).await
    }

    async fn get_packages_at(
        &self,
        proj: Project,
        date: i64,
    ) -> Result<Vec<PackageRow>, DatabaseError>
    {
        packages::get_packages_at(&self.0, proj, date).await
    }

    async fn get_package_id(
        &self,
        proj: Project,
        pkg: &str
    ) -> Result<Option<Package>, DatabaseError>
    {
        packages::get_package_id(&self.0, proj, pkg).await
    }

    async fn get_project_package_ids(
        &self,
        proj: &str,
        pkg: &str
    ) -> Result<Option<(Project, Package)>, DatabaseError>
    {
        packages::get_project_package_ids(&self.0, proj, pkg).await
    }

    async fn create_package(
        &self,
        owner: Owner,
        proj: Project,
        pkg: &str,
        pkg_data: &PackageDataPost,
        now: i64
    ) -> Result<(), DatabaseError>
    {
        packages::create_package(&self.0, owner, proj, pkg, pkg_data, now).await
    }

    async fn get_releases(
        &self,
        pkg: Package
    ) -> Result<Vec<ReleaseRow>, DatabaseError>
    {
        releases::get_releases(&self.0, pkg).await
    }

    async fn get_releases_at(
        &self,
        pkg: Package,
        date: i64
    ) -> Result<Vec<ReleaseRow>, DatabaseError>
    {
        releases::get_releases_at(&self.0, pkg, date).await
    }

    async fn get_release_id(
        &self,
        proj: Project,
        pkg: Package,
        release: &str
    ) -> Result<Option<Release>, DatabaseError>
    {
        releases::get_release_id(&self.0, proj, pkg, release).await
    }

    async fn get_project_package_release_ids(
        &self,
        projname: &str,
        pkgname: &str,
        release: &str
    ) -> Result<Option<(Project, Package, Release)>, DatabaseError> {
        releases::get_project_package_release_ids(
            &self.0,
            projname,
            pkgname,
            release
        ).await
    }

    async fn create_release(
        &self,
        owner: Owner,
        proj: Project,
        pkg: Package,
        version: &Version,
        now: i64
    ) -> Result<(), DatabaseError>
    {
        releases::create_release(&self.0, owner, proj, pkg, version, now).await
    }

    async fn get_files(
        &self,
        rel: Release
    ) -> Result<Vec<FileRow>, DatabaseError>
    {
        releases::get_files(&self.0, rel).await
    }

    async fn get_files_at(
        &self,
        rel: Release,
        date: i64
    ) -> Result<Vec<FileRow>, DatabaseError>
    {
        releases::get_files_at(&self.0, rel, date).await
    }

    async fn get_authors(
        &self,
        pkg_ver_id: i64
    ) -> Result<Users, CoreError>
    {
        get_authors(&self.0, pkg_ver_id).await
    }

/*
    async fn get_release_url(
        &self,
        pkg: Package
    ) -> Result<String, CoreError>
    {
        releases::get_release_url(&self.0, pkg).await
    }

     async fn get_release_version_url(
        &self,
        pkg: Package,
        version: &Version
    ) -> Result<String, CoreError>
    {
        releases::get_release_version_url(&self.0, pkg, version).await
    }
*/

    async fn add_file_url(
        &self,
        owner: Owner,
        proj: Project,
        release: Release,
        filename: &str,
        size: i64,
        sha256: &str,
        requires: Option<&str>,
        url: &str,
        now: i64
    ) -> Result<(), DatabaseError>
    {
        releases::add_file_url(
            &self.0,
            owner,
            proj,
            release,
            filename,
            size,
            sha256,
            requires,
            url,
            now
        ).await
    }

    async fn get_players(
        &self,
        proj: Project
    ) -> Result<Users, DatabaseError>
    {
        players::get_players(&self.0, proj).await
    }

    async fn add_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), DatabaseError>
    {
        players::add_player(&self.0, player, proj).await
    }

    async fn remove_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), DatabaseError>
    {
        players::remove_player(&self.0, player, proj).await
    }

    async fn get_image_url(
        &self,
        proj: Project,
        img_name: &str
    ) -> Result<Option<String>, DatabaseError>
    {
        images::get_image_url(&self.0, proj, img_name).await
    }

    async fn get_image_url_at(
        &self,
        proj: Project,
        img_name: &str,
        date: i64
    ) -> Result<Option<String>, DatabaseError>
    {
        images::get_image_url_at(&self.0, proj, img_name, date).await
    }

    async fn add_image_url(
        &self,
        owner: Owner,
        proj: Project,
        img_name: &str,
        url: &str,
        now: i64
    ) -> Result<(), DatabaseError>
    {
        images::add_image_url(&self.0, owner, proj, img_name, url, now).await
    }

    async fn get_tags(
        &self,
        proj: Project
    ) -> Result<Vec<String>, DatabaseError>
    {
        tags::get_tags(&self.0, proj).await
    }

    async fn get_tags_at(
        &self,
        proj: Project,
        date: i64
    ) -> Result<Vec<String>, DatabaseError>
    {
        tags::get_tags_at(&self.0, proj, date).await
    }

    async fn get_gallery(
        &self,
        proj: Project
    ) -> Result<Vec<GalleryImage>, DatabaseError> {
        images::get_gallery(&self.0, proj).await
    }

    async fn get_gallery_at(
        &self,
        proj: Project,
        date: i64
    ) -> Result<Vec<GalleryImage>, DatabaseError> {
        images::get_gallery_at(&self.0, proj, date).await
    }
}

// TODO: move this... somewhere else
async fn get_authors<'e, E>(
    ex: E,
    pkg_ver_id: i64
) -> Result<Users, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        Users {
            users: sqlx::query_scalar!(
                "
SELECT users.username
FROM users
JOIN authors
ON users.user_id = authors.user_id
WHERE authors.release_id = ?
ORDER BY users.username
                ",
                pkg_ver_id
            )
            .fetch_all(ex)
            .await?
        }
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(fixtures("users", "projects", "packages", "authors"))]
    async fn get_authors_ok(pool: Pool) {
        assert_eq!(
            get_authors(&pool, 2).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into()
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages", "authors"))]
    async fn get_authors_not_a_release(pool: Pool) {
        assert_eq!(
            get_authors(&pool, 0).await.unwrap(),
            Users { users: vec![] }
        );
    }
}
