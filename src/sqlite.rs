use axum::async_trait;
use sqlx::{
    Acquire, Executor,
    sqlite::Sqlite
};

use crate::{
    db::{DatabaseOperations, PackageRow, ProjectRow, VersionRow},
    errors::AppError,
    model::{GameData, ProjectID, ProjectDataPut, ProjectSummary, Readme, User, Users},
};

pub type Pool = sqlx::Pool<Sqlite>;

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
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

#[derive(Clone)]
pub struct SqliteDatabaseOperations {
}

#[async_trait]
impl DatabaseOperations<Sqlite> for SqliteDatabaseOperations {
    async fn get_project_id<'e, E>(
        &self,
        ex: E,
        project: &str
    ) -> Result<ProjectID, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_project_id(ex, project).await
    }

    async fn get_project_count<'e, E>(
        &self,
        ex: E
    ) -> Result<i32, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_project_count(ex).await 
    }

    async fn get_user_id<'e, E>(
        &self,
        ex: E,
        user: &str
    ) -> Result<i64, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_user_id(ex, user).await
    }

    async fn get_owners<'e, E>(
        &self,
        ex: E,
        proj_id: i64
    ) -> Result<Users, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_owners(ex, proj_id).await
    }

    async fn user_is_owner<'e, E>(
        &self,
        ex: E,
        user: &User,
        proj_id: i64
    ) -> Result<bool, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        user_is_owner(ex, user, proj_id).await
    }

    async fn add_owner<'e, E>(
        &self,
        ex: E,
        user_id: i64,
        proj_id: i64
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        add_owner(ex, user_id, proj_id).await
    }

    async fn remove_owner<'e, E>(
        &self,
        ex: E,
        user_id: i64,
        proj_id: i64
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        remove_owner(ex, user_id, proj_id).await
    }

    async fn has_owner<'e, E>(
        &self,
        ex: E,
        proj_id: i64,
    ) -> Result<bool, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        has_owner(ex, proj_id).await
    }

    async fn get_projects_start_window<'e, E>(
        &self,
        ex: E,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_projects_start_window(ex, limit).await
    }

    async fn get_projects_end_window<'e, E>(
        &self,
        ex: E,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_projects_end_window(ex, limit).await
    }

    async fn get_projects_after_window<'e, E>(
        &self,
        ex: E,
        name: &str,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_projects_after_window(ex, name, limit).await
    }

    async fn get_projects_before_window<'e, E>(
        &self,
        ex: E,
        name: &str,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_projects_before_window(ex, name, limit).await
    }

    async fn create_project<'e, E>(
        &self,
        ex: E,
        proj: &str,
        proj_data: &ProjectDataPut,
        game_title_sort_key: &str,
        now: &str
    ) -> Result<i64, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        create_project(ex, proj, proj_data, game_title_sort_key, now).await
    }

    async fn copy_project_revision<'e, E>(
        &self,
        ex: E,
        proj_id: i64
    ) -> Result<i64, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        copy_project_revision(ex, proj_id).await
    }

    async fn update_project<'e, E>(
        &self,
        ex: E,
        proj_id: i64,
        revision: i64,
        proj_data: &ProjectDataPut,
        now: &str
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        update_project(ex, proj_id, revision, proj_data, now).await
    }

    async fn get_project_row<'e, E>(
        &self,
        ex: E,
        proj_id: i64
    ) -> Result<ProjectRow, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_project_row(ex, proj_id).await
    }

    async fn get_project_row_revision<'a, A>(
        &self,
        conn: A,
        proj_id: i64,
        revision: u32
    ) -> Result<ProjectRow, AppError>
    where
        A: Acquire<'a, Database = Sqlite> + Send
    {
        get_project_row_revision(conn, proj_id, revision).await
    }

    async fn get_packages<'e, E>(
        &self,
        ex: E,
        proj_id: i64
    ) -> Result<Vec<PackageRow>, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_packages(ex, proj_id).await
    }

    async fn get_versions<'e, E>(
        &self,
        ex: E,
        pkg_id: i64
    ) -> Result<Vec<VersionRow>, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_versions(ex, pkg_id).await
    } 

    async fn get_package_url<'e, E>(
        &self,
        ex: E,
        pkg_id: i64
    ) -> Result<String, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_package_url(ex, pkg_id).await
    }

    async fn get_players<'e, E>(
        &self,
        ex: E,
        proj_id: i64
    ) -> Result<Users, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_players(ex, proj_id).await
    }

    async fn add_player<'e, E>(
        &self,
        ex: E,
        user_id: i64,
        proj_id: i64,
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        add_player(ex, user_id, proj_id).await
    }

    async fn remove_player<'e, E>(
        &self,
        ex: E,
        user_id: i64,
        proj_id: i64
    ) -> Result<(), AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        remove_player(ex, user_id, proj_id).await
    }

    async fn get_readme<'e, E>(
        &self,
        ex: E,
        proj_id: i64
    ) -> Result<Readme, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_readme(ex, proj_id).await
    }

    async fn get_readme_revision<'e, E>(
        &self,
        ex: E,
        proj_id: i64,
        revision: u32
    ) -> Result<Readme, AppError>
    where
        E: Executor<'e, Database = Sqlite>
    {
        get_readme_revision(ex, proj_id, revision).await
    }
}

pub async fn get_project_id<'e, E>(
    ex: E,
    project: &str
) -> Result<ProjectID, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT id
FROM projects
WHERE name = ?
        ",
        project
    )
    .fetch_optional(ex)
    .await?
    .map(ProjectID)
    .ok_or(AppError::NotAProject)
}

pub async fn get_project_count<'e, E>(
    ex: E
) -> Result<i32, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT COUNT(1)
FROM projects
            "
        )
        .fetch_one(ex)
        .await?
    )
}

pub async fn get_user_id<'e, E>(
    ex: E,
    user: &str
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT id
FROM users
WHERE username = ?
LIMIT 1
        ",
        user
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAUser)
}

pub async fn get_owners<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Users, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        Users {
            users: sqlx::query_scalar!(
                "
SELECT users.username
FROM users
JOIN owners
ON users.id = owners.user_id
JOIN projects
ON owners.project_id = projects.id
WHERE projects.id = ?
ORDER BY users.username
                ",
                proj_id
            )
            .fetch_all(ex)
            .await?
            .into_iter()
            .map(User)
            .collect()
        }
    )
}

pub async fn user_is_owner<'e, E>(
    ex: E,
    user: &User,
    proj_id: i64
) -> Result<bool, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 AS present
FROM owners
JOIN users
ON users.id = owners.user_id
WHERE users.username = ? AND owners.project_id = ?
LIMIT 1
            ",
            user.0,
            proj_id
        )
        .fetch_optional(ex)
        .await?
        .is_some()
    )
}

pub async fn add_owner<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query!(
        "
INSERT OR IGNORE INTO owners (
    user_id,
    project_id
)
VALUES (?, ?)
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn remove_owner<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query!(
        "
DELETE FROM owners
WHERE user_id = ?
    AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn has_owner<'e, E>(
    ex: E,
    proj_id: i64,
) -> Result<bool, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 AS present
FROM owners
WHERE project_id = ?
LIMIT 1
            ",
            proj_id
        )
        .fetch_optional(ex)
        .await?
        .is_some()
    )
}

pub async fn get_projects_start_window<'e, E>(
    ex: E,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_as!(
            ProjectRow,
            "
    SELECT
        name,
        description,
        revision,
        created_at,
        modified_at,
        game_title,
        game_title_sort,
        game_publisher,
        game_year
    FROM projects
    ORDER BY name COLLATE NOCASE ASC
    LIMIT ?
            ",
            limit
        )
        .fetch_all(ex)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn get_projects_end_window<'e, E>(
    ex: E,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_as!(
            ProjectRow,
            "
SELECT
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
FROM projects
ORDER BY name COLLATE NOCASE DESC
LIMIT ?
            ",
            limit
        )
        .fetch_all(ex)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn get_projects_after_window<'e, E>(
    ex: E,
    name: &str,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_as!(
            ProjectRow,
            "
SELECT
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
FROM projects
WHERE name > ?
ORDER BY name COLLATE NOCASE ASC
LIMIT ?
            ",
            name,
            limit
        )
        .fetch_all(ex)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn get_projects_before_window<'e, E>(
    ex: E,
    name: &str,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_as!(
            ProjectRow,
            "
SELECT
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
FROM projects
WHERE name < ?
ORDER BY name COLLATE NOCASE DESC
LIMIT ?
            ",
            name,
            limit
        )
        .fetch_all(ex)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn create_project<'e, E>(
    ex: E,
    proj: &str,
    proj_data: &ProjectDataPut,
    game_title_sort_key: &str,
    now: &str
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
INSERT INTO projects (
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
)
VALUES (?, ?, 1, ?, ?, ?, ?, ?, ?)
RETURNING id
            ",
            proj,
            proj_data.description,
            now,
            now,
            proj_data.game.title,
            game_title_sort_key,
            proj_data.game.publisher,
            proj_data.game.year
        )
        .fetch_one(ex)
        .await?
    )
}

pub async fn copy_project_revision<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
INSERT INTO projects_revisions
SELECT *
FROM projects
WHERE projects.id = ?
RETURNING revision
            ",
            proj_id
         )
        .fetch_one(ex)
        .await?
    )
}

pub async fn update_project<'e, E>(
    ex: E,
    proj_id: i64,
    revision: i64,
    proj_data: &ProjectDataPut,
    now: &str
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query!(
        "
UPDATE projects
SET
description = ?,
revision = ?,
modified_at = ?,
game_title = ?,
game_title_sort = ?,
game_publisher = ?,
game_year = ?
WHERE id = ?
        ",
        proj_data.description,
        revision,
        now,
        proj_data.game.title,
        proj_data.game.title_sort_key,
        proj_data.game.publisher,
        proj_data.game.year,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn get_project_row<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<ProjectRow, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query_as!(
        ProjectRow,
        "
SELECT
name,
description,
revision,
created_at,
modified_at,
game_title,
game_title_sort,
game_publisher,
game_year
FROM projects
WHERE id = ?
LIMIT 1
        ",
        proj_id
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAProject)
}

pub async fn get_project_row_revision<'a, A>(
    conn: A,
    proj_id: i64,
    revision: u32
) -> Result<ProjectRow, AppError>
where
    A: Acquire<'a, Database = sqlx::sqlite::Sqlite>
{
    let mut conn = conn.acquire().await?;

// TODO: check if a single UNION query is faster
    // check the revisions table
    let proj_row = sqlx::query_as!(
        ProjectRow,
        "
SELECT
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
FROM projects_revisions
WHERE id = ?
    AND revision = ?
LIMIT 1
        ",
        proj_id,
        revision
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::NotARevision);

    match proj_row {
        Ok(r) => Ok(r),
        Err(_) => {
            // check the current table
            sqlx::query_as!(
                ProjectRow,
                "
SELECT
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
FROM projects
WHERE id = ?
    AND revision = ?
LIMIT 1
                ",
                proj_id,
                revision
            )
            .fetch_optional(&mut *conn)
            .await?
            .ok_or(AppError::NotARevision)
        }
    }
}

pub async fn get_packages<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Vec<PackageRow>, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_as!(
            PackageRow,
            "
SELECT
    id,
    name
FROM packages
WHERE project_id = ?
ORDER BY name COLLATE NOCASE ASC
            ",
            proj_id
        )
       .fetch_all(ex)
       .await?
    )
}

pub async fn get_versions<'e, E>(
    ex: E,
    pkg_id: i64
) -> Result<Vec<VersionRow>, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        sqlx::query_as!(
            VersionRow,
            "
SELECT
    version,
    filename,
    url
FROM package_versions
WHERE package_id = ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC
            ",
            pkg_id
        )
        .fetch_all(ex)
        .await?
    )
}

// TODO: figure out how to order version_pre
pub async fn get_package_url<'e, E>(
    ex: E,
    pkg_id: i64
) -> Result<String, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT url
FROM package_versions
WHERE package_id = ?
ORDER BY
version_major DESC,
version_minor DESC,
version_patch DESC
LIMIT 1
        ",
        pkg_id
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAPackage)
}

pub async fn get_players<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Users, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    Ok(
        Users {
            users: sqlx::query_scalar!(
                "
SELECT users.username
FROM users
JOIN players
ON users.id = players.user_id
JOIN projects
ON players.project_id = projects.id
WHERE projects.id = ?
ORDER BY users.username
                ",
                proj_id
            )
            .fetch_all(ex)
            .await?
            .into_iter()
            .map(User)
            .collect()
        }
    )
}

pub async fn add_player<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query!(
        "
INSERT OR IGNORE INTO players (
    user_id,
    project_id
)
VALUES (?, ?)
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn remove_player<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query!(
        "
DELETE FROM players
WHERE user_id = ?
    AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn get_readme<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Readme, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query_as!(
        Readme,
        "
SELECT text
FROM readmes
WHERE project_id = ?
ORDER BY revision DESC
LIMIT 1
        ",
        proj_id
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAProject)
}

pub async fn get_readme_revision<'e, E>(
    ex: E,
    proj_id: i64,
    revision: u32
) -> Result<Readme, AppError>
where
    E: Executor<'e, Database = sqlx::sqlite::Sqlite>
{
    sqlx::query_as!(
        Readme,
        "
SELECT text
FROM readmes
WHERE project_id = ?
AND revision = ?
LIMIT 1
        ",
        proj_id,
        revision
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotARevision)
}


#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(fixtures("projects"))]
    async fn get_project_id_ok(pool: Pool) {
        assert_eq!(
            get_project_id(&pool, "test_game").await.unwrap(),
            ProjectID(42)
        );
    }

    #[sqlx::test(fixtures("projects"))]
    async fn get_project_id_not_a_project(pool: Pool) {
        assert_eq!(
            get_project_id(&pool, "bogus").await.unwrap_err(),
            AppError::NotAProject
        );
    }

    #[sqlx::test(fixtures("projects"))]
    async fn get_project_count_ok(pool: Pool) {
        assert_eq!(get_project_count(&pool).await.unwrap(), 2);
    }

    #[sqlx::test(fixtures("users"))]
    async fn get_user_id_ok(pool: Pool) {
        assert_eq!(get_user_id(&pool, "bob").await.unwrap(), 1);
    }

    #[sqlx::test(fixtures("users"))]
    async fn get_user_id_not_a_user(pool: Pool) {
        assert_eq!(
            get_user_id(&pool, "not_a_user").await.unwrap_err(),
            AppError::NotAUser
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

// TODO: can we tell when the project doesn't exist?
/*
    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn get_owners_not_a_project(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 0).await.unwrap_err(),
            AppError::NotAProject
        );
    }
*/

// TODO: can we tell when the project doesn't exist?

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        assert!(user_is_owner(&pool, &User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        assert!(!user_is_owner(&pool, &User("alice".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn add_owner_new(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
        add_owner(&pool, 2, 42).await.unwrap();
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into())
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn add_owner_existing(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
        add_owner(&pool, 1, 42).await.unwrap();
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

// TODO: add test for add_owner not a project
// TODO: add test for add_owner not a user

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn remove_owner_ok(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
        remove_owner(&pool, 1, 42).await.unwrap();
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![] }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn remove_owner_not_an_owner(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
        remove_owner(&pool, 2, 42).await.unwrap();
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

// TODO: add test for remove_owner not a project

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn has_owner_yes(pool: Pool) {
        assert!(has_owner(&pool, 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects"))]
    async fn has_owner_no(pool: Pool) {
        assert!(!has_owner(&pool, 42).await.unwrap());
    }

// TODO: add tests for create_project
// TODO: add tests for copy_project_revsion
// TODO: add tests for update_project

// TODO: add tests for get_projects_start_window
// TODO: add tests for get_projects_end_window
// TODO: add tests for get_projects_after_window
// TODO: add tests for get_projects_before_window

    #[sqlx::test(fixtures("projects"))]
    async fn get_project_row_ok(pool: Pool) {
        assert_eq!(
            get_project_row(&pool, 42).await.unwrap(),
            ProjectRow {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 1,
                created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                game_title: "A Game of Tests".into(),
                game_title_sort: "Game of Tests, A".into(),
                game_publisher: "Test Game Company".into(),
                game_year: "1979".into()
            }
        );
    }

    #[sqlx::test(fixtures("projects"))]
    async fn get_project_row_not_a_project(pool: Pool) {
        assert_eq!(
            get_project_row(&pool, 0).await.unwrap_err(),
            AppError::NotAProject
        );
    }

    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn get_project_row_revision_ok_current(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, 42, 1).await.unwrap(),
            ProjectRow {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 1,
                created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                game_title: "A Game of Tests".into(),
                game_title_sort: "Game of Tests, A".into(),
                game_publisher: "Test Game Company".into(),
                game_year: "1979".into()
            }
        );
    }

    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn get_project_revision_ok_old(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, 6, 1).await.unwrap(),
            ProjectRow {
                name: "a_game".into(),
                description: "Another game".into(),
                revision: 1,
                created_at: "2019-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2019-11-12T15:50:06.419538067+00:00".into(),
                game_title: "Some Otter Game".into(),
                game_title_sort: "Some Otter Game".into(),
                game_publisher: "Otters!".into(),
                game_year: "1993".into()
            }
        );
    }

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("projects"))]
    async fn get_project_revision_not_a_project(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, 0, 2).await.unwrap_err(),
            AppError::NotARevision
        );
    }

    #[sqlx::test(fixtures("projects"))]
    async fn get_project_revision_not_a_revision(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, 42, 2).await.unwrap_err(),
            AppError::NotARevision
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_packages_ok(pool: Pool) {
        assert_eq!(
            get_packages(&pool, 42).await.unwrap(),
            vec![
                PackageRow {
                    id: 1,
                    name: "a_package".into()
                },
                PackageRow {
                    id: 2,
                    name: "b_package".into()
                }
            ]
        );
    }

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_packages_not_a_project(pool: Pool) {
        assert_eq!(
            get_packages(&pool, 0).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_url_ok(pool: Pool) {
        assert_eq!(
            get_package_url(&pool, 1).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_url_not_a_package(pool: Pool) {
        assert_eq!(
            get_package_url(&pool, 0).await.unwrap_err(),
            AppError::NotAPackage
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_versions_ok(pool: Pool) {
        assert_eq!(
            get_versions(&pool, 1).await.unwrap(),
            vec![
                VersionRow {
                    version: "1.2.4".into(),
                    filename: "a_package-1.2.4".into(),
                    url: "https://example.com/a_package-1.2.4".into()
                },
                VersionRow {
                    version: "1.2.3".into(),
                    filename: "a_package-1.2.3".into(),
                    url: "https://example.com/a_package-1.2.3".into()
                }
            ]
        );
    }

// TODO: can we tell when the package doesn't exist?
    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_versions_not_a_package(pool: Pool) {
        assert_eq!(
            get_versions(&pool, 0).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
    async fn get_players_ok(pool: Pool) {
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into())
                ]
            }
        );
    }

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("projects", "players"))]
    async fn get_players_not_a_project(pool: Pool) {
        assert_eq!(
            get_players(&pool, 0).await.unwrap(),
            Users { users: vec![] }
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
    async fn add_player_new(pool: Pool) {
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                ]
            }
        );
        add_player(&pool, 3, 42).await.unwrap();
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                    User("chuck".into())
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
    async fn add_player_existing(pool: Pool) {
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                ]
            }
        );
        add_player(&pool, 2, 42).await.unwrap();
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                ]
            }
        );
    }

// TODO: add test for add_player not a project
// TODO: add test for add_player not a user

    #[sqlx::test(fixtures("projects", "players"))]
    async fn remove_player_existing(pool: Pool) {
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                ]
            }
        );
        remove_player(&pool, 2, 42).await.unwrap();
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("bob".into()),
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn remove_player_not_a_player(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
        remove_owner(&pool, 2, 42).await.unwrap();
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

// TODO: add test for remove_player not a project

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_ok(pool: Pool) {
        assert_eq!(
            get_readme(&pool, 42).await.unwrap(),
            Readme { text: "third try".into() }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_not_a_project(pool: Pool) {
        assert_eq!(
            get_readme(&pool, 0).await.unwrap_err(),
            AppError::NotAProject
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_revision_ok(pool: Pool) {
        assert_eq!(
            get_readme_revision(&pool, 42, 2).await.unwrap(),
            Readme { text: "second try".into() }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_revision_bad(pool: Pool) {
        assert_eq!(
            get_readme_revision(&pool, 42, 4).await.unwrap_err(),
            AppError::NotARevision
        );
    }
}
