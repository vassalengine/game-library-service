use serde::Deserialize;
use sqlx::Executor;

use crate::{
    errors::AppError,
    model::{GameData, ProjectID, ProjectDataPut, ProjectSummary, Readme, User, Users},
};

type Database = sqlx::sqlite::Sqlite;
pub type Pool = sqlx::Pool<Database>;

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

pub async fn get_project_id(
    db: &Pool,
    project: &str
) -> Result<ProjectID, AppError>
{
    sqlx::query_scalar!(
        "
SELECT id
FROM projects
WHERE name = ?
        ",
        project
    )
    .fetch_optional(db)
    .await?
    .map(ProjectID)
    .ok_or(AppError::NotAProject)
}

pub async fn get_project_count(
    db: &Pool
) -> Result<i32, AppError>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT COUNT(1)
FROM projects
            "
        )
        .fetch_one(db)
        .await?
    )
}

pub async fn get_user_id(
    db: &Pool,
    user: &str
) -> Result<i64, AppError> {
    sqlx::query_scalar!(
        "
SELECT id
FROM users
WHERE username = ?
LIMIT 1
        ",
        user
    )
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotAUser)
}

pub async fn get_owners(
    db: &Pool,
    proj_id: i64
) -> Result<Users, AppError>
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
            .fetch_all(db)
            .await?
            .into_iter()
            .map(User)
            .collect()
        }
    )
}

pub async fn user_is_owner(
    db: &Pool,
    user: &User,
    proj_id: i64
) -> Result<bool, AppError>
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
        .fetch_optional(db)
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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

pub async fn get_projects_start_window(
    db: &Pool,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
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
        .fetch_all(db)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn get_projects_end_window(
    db: &Pool,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
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
        .fetch_all(db)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn get_projects_after_window(
    db: &Pool,
    name: &str,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
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
        .fetch_all(db)
        .await?
        .into_iter()
        .map(ProjectSummary::from)
        .collect()
    )
}

pub async fn get_projects_before_window(
    db: &Pool,
    name: &str,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
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
        .fetch_all(db)
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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

pub async fn get_project_row(
    db: &Pool,
    proj_id: i64,
) -> Result<ProjectRow, AppError>
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
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotAProject)
}

pub async fn get_project_row_revision(
    db: &Pool,
    proj_id: i64,
    revision: u32
) -> Result<ProjectRow, AppError>
{
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
    .fetch_optional(db)
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
            .fetch_optional(db)
            .await?
            .ok_or(AppError::NotARevision)
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PackageRow {
    pub id: i64,
    pub name: String,
//    description: String
}

pub async fn get_packages(
    db: &Pool,
    proj_id: i64
) -> Result<Vec<PackageRow>, AppError> {
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
       .fetch_all(db)
       .await?
    )
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

pub async fn get_versions(
    db: &Pool,
    pkg_id: i64
) -> Result<Vec<VersionRow>, AppError> {
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
        .fetch_all(db)
        .await?
    )
}

// TODO: figure out how to order version_pre
pub async fn get_package_url(
    db: &Pool,
    pkg_id: i64
) -> Result<String, AppError>
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
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotAPackage)
}

pub async fn get_players(
    db: &Pool,
    proj_id: i64
) -> Result<Users, AppError>
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
            .fetch_all(db)
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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

pub async fn get_readme(
    db: &Pool,
    proj_id: i64
) -> Result<Readme, AppError>
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
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotAProject)
}

pub async fn get_readme_revision(
    db: &Pool,
    proj_id: i64,
    revision: u32
) -> Result<Readme, AppError>
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
    .fetch_optional(db)
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

// TODO: add tests for add_owner
// TODO: add tests for remove_owner
// TODO: add tests for has_owner

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

// TODO: add tests for add_owner
// TODO: add tests for remove_owner

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
