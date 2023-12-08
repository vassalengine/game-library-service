use axum::async_trait;
use sqlx::{
    Acquire, Database, Executor,
    sqlite::Sqlite
};

use crate::{
    db::{DatabaseClient, PackageRow, ProjectRow, VersionRow},
    errors::AppError,
    model::{GameData, ProjectID, ProjectDataPut, ProjectSummary, Readme, User, Users},
    version::Version
};

pub type Pool = sqlx::Pool<Sqlite>;

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

#[derive(Clone)]
pub struct SqlxDatabaseClient<DB: Database>(pub sqlx::Pool<DB>);

#[async_trait]
impl DatabaseClient for SqlxDatabaseClient<Sqlite> {
    async fn get_project_id(
        &self,
        project: &str
    ) -> Result<ProjectID, AppError>
    {
        get_project_id(&self.0, project).await
    }

    async fn get_project_count(
        &self,
    ) -> Result<i32, AppError>
    {
        get_project_count(&self.0).await
    }

    async fn get_user_id(
        &self,
        user: &str
    ) -> Result<i64, AppError>
    {
        get_user_id(&self.0, user).await
    }

    async fn get_owners(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        get_owners(&self.0, proj_id).await
    }

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: i64
    ) -> Result<bool, AppError>
    {
        user_is_owner(&self.0, user, proj_id).await
    }

    async fn add_owner(
        &self,
        user_id: i64,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        add_owner(&self.0, user_id, proj_id).await
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        add_owners(&self.0, owners, proj_id).await
    }

    async fn remove_owner(
        &self,
        user_id: i64,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        remove_owner(&self.0, user_id, proj_id).await
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        remove_owners(&self.0, owners, proj_id).await
    }

    async fn has_owner(
        &self,
        proj_id: i64,
    ) -> Result<bool, AppError>
    {
        has_owner(&self.0, proj_id).await
    }

    async fn get_projects_start_window(
        &self,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        get_projects_start_window(&self.0, limit).await
    }

    async fn get_projects_end_window(
        &self,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        get_projects_end_window(&self.0, limit).await
    }

    async fn get_projects_after_window(
        &self,
        name: &str,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        get_projects_after_window(&self.0, name, limit).await
    }

    async fn get_projects_before_window(
        &self,
        name: &str,
        limit: u32
    ) -> Result<Vec<ProjectSummary>, AppError>
    {
        get_projects_before_window(&self.0, name, limit).await
    }

    async fn create_project(
        &self,
        user: &User,
        proj: &str,
        proj_data: &ProjectDataPut,
        now: &str
    ) -> Result<(), AppError>
    {
        create_project(&self.0, user, proj, proj_data, now).await
    }

    async fn copy_project_revision(
        &self,
        proj_id: i64
    ) -> Result<i64, AppError>
    {
        copy_project_revision(&self.0, proj_id).await
    }

    async fn update_project(
        &self,
        proj_id: i64,
        proj_data: &ProjectDataPut,
        now: &str
    ) -> Result<(), AppError>
    {
        update_project(&self.0, proj_id, proj_data, now).await
    }

    async fn get_project_row(
        &self,
        proj_id: i64
    ) -> Result<ProjectRow, AppError>
    {
        get_project_row(&self.0, proj_id).await
    }

    async fn get_project_row_revision(
        &self,
        proj_id: i64,
        revision: u32
    ) -> Result<ProjectRow, AppError>
    {
        get_project_row_revision(&self.0, proj_id, revision).await
    }

    async fn get_packages(
        &self,
        proj_id: i64
    ) -> Result<Vec<PackageRow>, AppError>
    {
        get_packages(&self.0, proj_id).await
    }

    async fn get_versions(
        &self,
        pkg_id: i64
    ) -> Result<Vec<VersionRow>, AppError>
    {
        get_versions(&self.0, pkg_id).await
    }

    async fn get_package_url(
        &self,
        pkg_id: i64
    ) -> Result<String, AppError>
    {
        get_package_url(&self.0, pkg_id).await
    }

     async fn get_package_version_url(
        &self,
        pkg_id: i64,
        version: &Version
    ) -> Result<String, AppError>
    {
        get_package_version_url(&self.0, pkg_id, version).await
    }

    async fn get_players(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        get_players(&self.0, proj_id).await
    }

    async fn add_player(
        &self,
        player: &User,
        proj_id: i64,
    ) -> Result<(), AppError>
    {
        add_player(&self.0, player, proj_id).await
    }

    async fn remove_player(
        &self,
        player: &User,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        remove_player(&self.0, player, proj_id).await
    }

    async fn get_readme(
        &self,
        proj_id: i64
    ) -> Result<Readme, AppError>
    {
        get_readme(&self.0, proj_id).await
    }

    async fn get_readme_revision(
        &self,
        proj_id: i64,
        revision: u32
    ) -> Result<Readme, AppError>
    {
        get_readme_revision(&self.0, proj_id, revision).await
    }
}

async fn get_project_id<'e, E>(
    ex: E,
    project: &str
) -> Result<ProjectID, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT project_id
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

async fn get_project_count<'e, E>(
    ex: E
) -> Result<i32, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_user_id<'e, E>(
    ex: E,
    user: &str
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT user_id
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

async fn get_owners<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Users, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        Users {
            users: sqlx::query_scalar!(
                "
SELECT users.username
FROM users
JOIN owners
ON users.user_id = owners.user_id
JOIN projects
ON owners.project_id = projects.project_id
WHERE projects.project_id = ?
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

async fn user_is_owner<'e, E>(
    ex: E,
    user: &User,
    proj_id: i64
) -> Result<bool, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 AS present
FROM owners
JOIN users
ON users.user_id = owners.user_id
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

async fn add_owner<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn add_owners<'a, A>(
    conn: A,
    owners: &Users,
    proj_id: i64
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    for owner in &owners.users {
        // get user id of new owner
        let owner_id = get_user_id(&mut *tx, &owner.0).await?;
        // associate new owner with the project
        add_owner(&mut *tx, owner_id, proj_id).await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn remove_owner<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn remove_owners<'a, A>(
    conn: A,
    owners: &Users,
    proj_id: i64
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    for owner in &owners.users {
        // get user id of owner
        let owner_id = get_user_id(&mut *tx, &owner.0).await?;
        // remove old owner from the project
        remove_owner(&mut *tx, owner_id, proj_id).await?;
    }

    // prevent removal of last owner
    if !has_owner(&mut *tx, proj_id).await? {
        return Err(AppError::CannotRemoveLastOwner);
    }

    tx.commit().await?;

    Ok(())
}

async fn has_owner<'e, E>(
    ex: E,
    proj_id: i64,
) -> Result<bool, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_projects_start_window<'e, E>(
    ex: E,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_projects_end_window<'e, E>(
    ex: E,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_projects_after_window<'e, E>(
    ex: E,
    name: &str,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_projects_before_window<'e, E>(
    ex: E,
    name: &str,
    limit: u32
) -> Result<Vec<ProjectSummary>, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn create_project_row<'e, E>(
    ex: E,
    proj: &str,
    proj_data: &ProjectDataPut,
    now: &str
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>
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
RETURNING project_id
            ",
            proj,
            proj_data.description,
            now,
            now,
            proj_data.game.title,
            proj_data.game.title_sort_key,
            proj_data.game.publisher,
            proj_data.game.year
        )
        .fetch_one(ex)
        .await?
    )
}

async fn create_project<'a, A>(
    conn: A,
    user: &User,
    proj: &str,
    proj_data: &ProjectDataPut,
    now: &str
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    let proj_id = create_project_row(
        &mut *tx,
        proj,
        proj_data,
        &now
    ).await?;

    // get user id of new owner
    let owner_id = get_user_id(&mut *tx, &user.0).await?;

    // associate new owner with the project
    add_owner(&mut *tx, owner_id, proj_id).await?;

    tx.commit().await?;

    Ok(())
}

async fn copy_project_revision<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
INSERT INTO projects_revisions
SELECT *
FROM projects
WHERE projects.project_id = ?
RETURNING revision
            ",
            proj_id
         )
        .fetch_one(ex)
        .await?
    )
}

async fn update_project_row<'e, E>(
    ex: E,
    proj_id: i64,
    revision: i64,
    proj_data: &ProjectDataPut,
    now: &str
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
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
WHERE project_id = ?
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

async fn update_project<'a, A>(
    conn: A,
    proj_id: i64,
    proj_data: &ProjectDataPut,
    now: &str
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // archive the previous revision
    let revision = 1 + copy_project_revision(&mut *tx, proj_id).await?;

    // update to the current revision
    update_project_row(&mut *tx, proj_id, revision, proj_data, now).await?;

    tx.commit().await?;

    Ok(())
}

async fn get_project_row<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<ProjectRow, AppError>
where
    E: Executor<'e, Database = Sqlite>
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
WHERE project_id = ?
LIMIT 1
        ",
        proj_id
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAProject)
}

async fn get_project_row_revision<'a, A>(
    conn: A,
    proj_id: i64,
    revision: u32
) -> Result<ProjectRow, AppError>
where
    A: Acquire<'a, Database = Sqlite>
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
WHERE project_id = ?
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
WHERE project_id = ?
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

async fn get_packages<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Vec<PackageRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_as!(
            PackageRow,
            "
SELECT
    package_id,
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

async fn get_versions<'e, E>(
    ex: E,
    pkg_id: i64
) -> Result<Vec<VersionRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>
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
async fn get_package_url<'e, E>(
    ex: E,
    pkg_id: i64
) -> Result<String, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_package_version_url<'e, E>(
    ex: E,
    pkg_id: i64,
    version: &Version
) -> Result<String, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    let pre = version.pre.as_deref().unwrap_or("");
    let build = version.build.as_deref().unwrap_or("");

    sqlx::query_scalar!(
        "
SELECT url
FROM package_versions
WHERE package_id = ?
    AND version_major = ?
    AND version_minor = ?
    AND version_patch = ?
    AND version_pre = ?
    AND version_build = ?
LIMIT 1
        ",
        pkg_id,
        version.major,
        version.minor,
        version.patch,
        pre,
        build
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAVersion)
}

async fn get_players<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Users, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        Users {
            users: sqlx::query_scalar!(
                "
SELECT users.username
FROM users
JOIN players
ON users.user_id = players.user_id
JOIN projects
ON players.project_id = projects.project_id
WHERE projects.project_id = ?
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

async fn add_player_id<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64,
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn add_player<'a, A>(
    conn: A,
    player: &User,
    proj_id: i64
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // get user id of new player
    let player_id = get_user_id(&mut *tx, &player.0).await?;
    // associate new player with the project
    add_player_id(&mut *tx, player_id, proj_id).await?;

    tx.commit().await?;

    Ok(())
}

async fn remove_player_id<'e, E>(
    ex: E,
    user_id: i64,
    proj_id: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn remove_player<'a, A>(
    conn: A,
    player: &User,
    proj_id: i64
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // get user id of player
    let player_id = get_user_id(&mut *tx, &player.0).await?;
    // remove player from the project
    remove_player_id(&mut *tx, player_id, proj_id).await?;

    tx.commit().await?;

    Ok(())
}

async fn get_readme<'e, E>(
    ex: E,
    proj_id: i64
) -> Result<Readme, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

async fn get_readme_revision<'e, E>(
    ex: E,
    proj_id: i64,
    revision: u32
) -> Result<Readme, AppError>
where
    E: Executor<'e, Database = Sqlite>
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

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

// TODO: can we tell when the project doesn't exist?
/*
    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
    async fn get_owners_not_a_project(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 0).await.unwrap_err(),
            AppError::NotAProject
        );
    }
*/

// TODO: can we tell when the project doesn't exist?

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        assert!(user_is_owner(&pool, &User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        assert!(!user_is_owner(&pool, &User("alice".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
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

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
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

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
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

    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
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
    #[sqlx::test(fixtures("projects", "users", "one_owner"))]
    async fn has_owner_yes(pool: Pool) {
        assert!(has_owner(&pool, 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects"))]
    async fn has_owner_no(pool: Pool) {
        assert!(!has_owner(&pool, 42).await.unwrap());
    }

    #[sqlx::test(fixtures("users"))]
    async fn create_project_ok(pool: Pool) {
        let user = User("bob".into());
        let row = ProjectRow {
            name: "test_game".into(),
            description: "Brian's Trademarked Game of Being a Test Case".into(),
            revision: 1,
            created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
            modified_at: "2023-11-12T15:50:06.419538067+00:00".into(),
            game_title: "A Game of Tests".into(),
            game_title_sort: "Game of Tests, A".into(),
            game_publisher: "Test Game Company".into(),
            game_year: "1979".into()
        };

        let cdata = ProjectDataPut {
            description: row.description.clone(),
            tags: vec![],
            game: GameData {
                title: row.game_title.clone(),
                title_sort_key: row.game_title_sort.clone(),
                publisher: row.game_publisher.clone(),
                year: row.game_year.clone()
            }
        };

        assert_eq!(
            get_project_id(&pool, &row.name).await.unwrap_err(),
            AppError::NotAProject
        );

        create_project(
            &pool,
            &user,
            &row.name,
            &cdata,
            &row.created_at
        ).await.unwrap();

        let proj_id = get_project_id(&pool, &row.name).await.unwrap();

        assert_eq!(
            get_project_row(&pool, proj_id.0).await.unwrap(),
            row
        );
    }

// TODO: add tests for create_project
/*
    #[sqlx::test(fixtures("users"))]
    async fn create_project_bad(pool: Pool) {
    }
*/

// TODO: add tests for copy_project_revsion
// TODO: add tests for update_project

    fn fake_project_summary(name: String) -> ProjectSummary {
        ProjectSummary {
            name,
            description: "".into(),
            revision: 1,
            created_at: "".into(),
            modified_at: "".into(),
            tags: vec![],
            game: GameData {
                title: "".into(),
                title_sort_key: "".into(),
                publisher: "".into(),
                year: "".into()
            }
        }
    }

    #[sqlx::test]
    async fn get_projects_start_window_empty(pool: Pool) {
        assert_eq!(
            get_projects_start_window(&pool, 3).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_start_window_not_all(pool: Pool) {
        assert_eq!(
            get_projects_start_window(&pool, 3).await.unwrap(),
            "abc".chars()
                .map(|c| fake_project_summary(c.into()))
                .collect::<Vec<ProjectSummary>>()
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_start_window_past_end(pool: Pool) {
        assert_eq!(
            get_projects_start_window(&pool, 5).await.unwrap(),
            "abcd".chars()
                .map(|c| fake_project_summary(c.into()))
                .collect::<Vec<ProjectSummary>>()
        );
    }

    #[sqlx::test]
    async fn get_projects_end_window_empty(pool: Pool) {
        assert_eq!(
            get_projects_end_window(&pool, 3).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_end_window_not_all(pool: Pool) {
        assert_eq!(
            get_projects_end_window(&pool, 3).await.unwrap(),
            "dcb".chars()
                .map(|c| fake_project_summary(c.into()))
                .collect::<Vec<ProjectSummary>>()
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_end_window_past_start(pool: Pool) {
        assert_eq!(
            get_projects_end_window(&pool, 5).await.unwrap(),
            "dcba".chars()
                .map(|c| fake_project_summary(c.into()))
                .collect::<Vec<ProjectSummary>>()
        );
    }

    #[sqlx::test]
    async fn get_projects_after_window_empty(pool: Pool) {
        assert_eq!(
            get_projects_after_window(&pool, "a", 3).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_after_window_not_all(pool: Pool) {
        assert_eq!(
            get_projects_after_window(&pool, "b", 3).await.unwrap(),
            "cd".chars()
                .map(|c| fake_project_summary(c.into()))
                .collect::<Vec<ProjectSummary>>()
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_after_window_past_end(pool: Pool) {
        assert_eq!(
            get_projects_after_window(&pool, "d", 3).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test]
    async fn get_projects_before_window_empty(pool: Pool) {
        assert_eq!(
            get_projects_before_window(&pool, "d", 3).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_before_window_not_all(pool: Pool) {
        assert_eq!(
            get_projects_before_window(&pool, "c", 3).await.unwrap(),
            "ba".chars()
                .map(|c| fake_project_summary(c.into()))
                .collect::<Vec<ProjectSummary>>()
        );
    }

    #[sqlx::test(fixtures("proj_window"))]
    async fn get_projects_before_window_past_start(pool: Pool) {
        assert_eq!(
            get_projects_before_window(&pool, "a", 3).await.unwrap(),
            vec![]
        );
    }

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

    #[sqlx::test(fixtures("projects", "users", "two_owners"))]
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

    #[sqlx::test(fixtures("projects", "users", "two_owners"))]
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
                    package_id: 1,
                    name: "a_package".into()
                },
                PackageRow {
                    package_id: 2,
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

    #[sqlx::test(fixtures("projects", "users", "players"))]
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
    #[sqlx::test(fixtures("projects", "users", "players"))]
    async fn get_players_not_a_project(pool: Pool) {
        assert_eq!(
            get_players(&pool, 0).await.unwrap(),
            Users { users: vec![] }
        );
    }

    #[sqlx::test(fixtures("projects", "users", "players"))]
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
        add_player(&pool, &User("chuck".into()), 42).await.unwrap();
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

    #[sqlx::test(fixtures("projects", "users", "players"))]
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
        add_player(&pool, &User("alice".into()), 42).await.unwrap();
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

    #[sqlx::test(fixtures("projects", "users", "players"))]
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
        remove_player(&pool, &User("alice".into()), 42).await.unwrap();
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("bob".into()),
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "users", "players"))]
    async fn remove_player_not_a_player(pool: Pool) {
        assert_eq!(
            get_players(&pool, 42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                ]
            }
        );
        remove_player(&pool, &User("chuck".into()), 42).await.unwrap();
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
