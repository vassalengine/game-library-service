use axum::async_trait;
use sqlx::{
    Database, Executor,
    sqlite::Sqlite
};

mod images;
mod packages;
mod players;
mod projects;
mod releases;
mod users;

use crate::{
    core::CoreError,
    db::{DatabaseClient, PackageRow, ProjectRow, ProjectSummaryRow, ReleaseRow},
    model::{Owner, Package, PackageDataPost, Project, ProjectDataPatch, ProjectDataPost, User, Users},
    pagination::{Direction, SortBy},
    time::rfc3339_to_nanos,
    version::Version
};

pub type Pool = sqlx::Pool<Sqlite>;

#[derive(Clone)]
pub struct SqlxDatabaseClient<DB: Database>(pub sqlx::Pool<DB>);

#[async_trait]
impl DatabaseClient for SqlxDatabaseClient<Sqlite> {
    async fn get_project_id(
        &self,
        name: &str
    ) -> Result<Project, CoreError>
    {
        projects::get_project_id(&self.0, name).await
    }

    async fn get_projects_count(
        &self,
    ) -> Result<i64, CoreError>
    {
        projects::get_projects_count(&self.0).await
    }

    async fn get_projects_query_count(
        &self,
        query: &str
    ) -> Result<i64, CoreError>
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
    ) -> Result<bool, CoreError>
    {
        users::user_is_owner(&self.0, user, proj).await
    }

    async fn add_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<(), CoreError>
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
    ) -> Result<(), CoreError>
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
    ) -> Result<bool, CoreError>
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
    ) -> Result<(), CoreError>
    {
        projects::create_project(&self.0, user, proj, proj_data, now).await
    }

    async fn update_project(
        &self,
        owner: Owner,
        proj: Project,
        proj_data: &ProjectDataPatch,
        now: i64
    ) -> Result<(), CoreError>
    {
        projects::update_project(&self.0, owner, proj, proj_data, now).await
    }

    async fn get_project_row(
        &self,
        proj: Project
    ) -> Result<ProjectRow, CoreError>
    {
        projects::get_project_row(&self.0, proj).await
    }

    async fn get_project_row_revision(
        &self,
        proj: Project,
        revision: i64
    ) -> Result<ProjectRow, CoreError>
    {
        projects::get_project_row_revision(&self.0, proj, revision).await
    }

    async fn get_packages(
        &self,
        proj: Project
    ) -> Result<Vec<PackageRow>, CoreError>
    {
        packages::get_packages(&self.0, proj).await
    }

    async fn get_packages_at(
        &self,
        proj: Project,
        date: i64,
    ) -> Result<Vec<PackageRow>, CoreError>
    {
        packages::get_packages_at(&self.0, proj, date).await
    }

    async fn create_package(
        &self,
        owner: Owner,
        proj: Project,
        pkg: &str,
        pkg_data: &PackageDataPost,
        now: i64
    ) -> Result<(), CoreError>
    {
        packages::create_package(&self.0, owner, proj, pkg, pkg_data, now).await
    }

    async fn get_releases(
        &self,
        pkg: Package
    ) -> Result<Vec<ReleaseRow>, CoreError>
    {
        releases::get_releases(&self.0, pkg).await
    }

    async fn get_releases_at(
        &self,
        pkg: Package,
        date: i64
    ) -> Result<Vec<ReleaseRow>, CoreError>
    {
        releases::get_releases_at(&self.0, pkg, date).await
    }

    async fn get_authors(
        &self,
        pkg_ver_id: i64
    ) -> Result<Users, CoreError>
    {
        get_authors(&self.0, pkg_ver_id).await
    }

    async fn get_package_url(
        &self,
        pkg: Package
    ) -> Result<String, CoreError>
    {
        releases::get_package_url(&self.0, pkg).await
    }

     async fn get_release_url(
        &self,
        pkg: Package,
        version: &Version
    ) -> Result<String, CoreError>
    {
        releases::get_release_url(&self.0, pkg, version).await
    }

    async fn get_players(
        &self,
        proj: Project
    ) -> Result<Users, CoreError>
    {
        players::get_players(&self.0, proj).await
    }

    async fn add_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), CoreError>
    {
        players::add_player(&self.0, player, proj).await
    }

    async fn remove_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), CoreError>
    {
        players::remove_player(&self.0, player, proj).await
    }

    async fn get_image_url(
        &self,
        proj: Project,
        img_name: &str
    ) -> Result<String, CoreError>
    {
        images::get_image_url(&self.0, proj, img_name).await
    }

    async fn get_image_url_at(
        &self,
        proj: Project,
        img_name: &str,
        date: i64
    ) -> Result<String, CoreError>
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
    ) -> Result<(), CoreError>
    {
        images::add_image_url(&self.0, owner, proj, img_name, url, now).await
    }
}

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

// TODO: can we tell when the package version doesn't exist?
    #[sqlx::test(fixtures("users", "projects", "packages", "authors"))]
    async fn get_authors_not_a_release(pool: Pool) {
        assert_eq!(
            get_authors(&pool, 0).await.unwrap(),
            Users { users: vec![] }
        );
    }
}
