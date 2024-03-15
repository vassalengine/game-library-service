use axum::async_trait;
use sqlx::{
    Acquire, Database, Encode, Executor, QueryBuilder, Type,
    sqlite::Sqlite
};
use serde::Deserialize;
use std::cmp::Ordering;

use crate::{
    db::{DatabaseClient, PackageRow, ProjectRow, ProjectSummaryRow, ReleaseRow},
    errors::AppError,
    model::{Owner, Package, PackageDataPost, Project, ProjectDataPatch, ProjectDataPost, User, Users},
    pagination::{Direction, SortBy},
    time::rfc3339_to_nanos,
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
        name: &str
    ) -> Result<Project, AppError>
    {
        get_project_id(&self.0, name).await
    }

    async fn get_projects_count(
        &self,
    ) -> Result<i64, AppError>
    {
        get_projects_count(&self.0).await
    }

    async fn get_projects_query_count(
        &self,
        query: &str
    ) -> Result<i64, AppError>
    {
        get_projects_query_count(&self.0, query).await
    }

    async fn get_user_id(
        &self,
        username: &str
    ) -> Result<User, AppError>
    {
        get_user_id(&self.0, username).await
    }

    async fn get_owners(
        &self,
        proj: Project
    ) -> Result<Users, AppError>
    {
        get_owners(&self.0, proj).await
    }

    async fn user_is_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<bool, AppError>
    {
        user_is_owner(&self.0, user, proj).await
    }

    async fn add_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<(), AppError>
    {
        add_owner(&self.0, user, proj).await
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), AppError>
    {
        add_owners(&self.0, owners, proj).await
    }

    async fn remove_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<(), AppError>
    {
        remove_owner(&self.0, user, proj).await
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), AppError>
    {
        remove_owners(&self.0, owners, proj).await
    }

    async fn has_owner(
        &self,
        proj: Project
    ) -> Result<bool, AppError>
    {
        has_owner(&self.0, proj).await
    }

    async fn get_projects_end_window(
        &self,
        sort_by: SortBy,
        dir: Direction,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, AppError>
    {
        get_projects_end_window(&self.0, sort_by, dir, limit).await
    }

    async fn get_projects_query_end_window(
        &self,
        query: &str,
        sort_by: SortBy,
        dir: Direction,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, AppError>
    {
        get_projects_query_end_window(&self.0, query, sort_by, dir, limit).await
    }

    async fn get_projects_mid_window(
        &self,
        sort_by: SortBy,
        dir: Direction,
        field: &str,
        id: u32,
        limit: u32
    ) -> Result<Vec<ProjectSummaryRow>, AppError>
    {
        match sort_by {
            SortBy::CreationTime |
            SortBy::ModificationTime => get_projects_mid_window(
                &self.0,
                sort_by,
                dir,
                &rfc3339_to_nanos(field)?,
                id,
                limit
            ).await,
            _ => get_projects_mid_window(
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
    ) -> Result<Vec<ProjectSummaryRow>, AppError>
    {
        match sort_by {
            SortBy::CreationTime |
            SortBy::ModificationTime => get_projects_query_mid_window(
                &self.0,
                query,
                sort_by,
                dir,
                &rfc3339_to_nanos(field)?,
                id,
                limit
            ).await,
            SortBy::Relevance => get_projects_query_mid_window(
                &self.0,
                query,
                sort_by,
                dir,
                &field.parse::<f64>().map_err(|_| AppError::MalformedQuery)?,
                id,
                limit
            ).await,
            _ => get_projects_query_mid_window(
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
    ) -> Result<(), AppError>
    {
        create_project(&self.0, user, proj, proj_data, now).await
    }

    async fn update_project(
        &self,
        owner: Owner,
        proj: Project,
        proj_data: &ProjectDataPatch,
        now: i64
    ) -> Result<(), AppError>
    {
        update_project(&self.0, owner, proj, proj_data, now).await
    }

    async fn get_project_row(
        &self,
        proj: Project
    ) -> Result<ProjectRow, AppError>
    {
        get_project_row(&self.0, proj).await
    }

    async fn get_project_row_revision(
        &self,
        proj: Project,
        revision: i64
    ) -> Result<ProjectRow, AppError>
    {
        get_project_row_revision(&self.0, proj, revision).await
    }

    async fn get_packages(
        &self,
        proj: Project
    ) -> Result<Vec<PackageRow>, AppError>
    {
        get_packages(&self.0, proj).await
    }

    async fn get_packages_at(
        &self,
        proj: Project,
        date: i64,
    ) -> Result<Vec<PackageRow>, AppError>
    {
        get_packages_at(&self.0, proj, date).await
    }

    async fn create_package(
        &self,
        owner: Owner,
        proj: Project,
        pkg: &str,
        pkg_data: &PackageDataPost,
        now: i64
    ) -> Result<(), AppError>
    {
        create_package(&self.0, owner, proj, pkg, pkg_data, now).await
    }

    async fn get_releases(
        &self,
        pkg: Package
    ) -> Result<Vec<ReleaseRow>, AppError>
    {
        get_releases(&self.0, pkg).await
    }

    async fn get_releases_at(
        &self,
        pkg: Package,
        date: i64
    ) -> Result<Vec<ReleaseRow>, AppError>
    {
        get_releases_at(&self.0, pkg, date).await
    }

    async fn get_authors(
        &self,
        pkg_ver_id: i64
    ) -> Result<Users, AppError>
    {
        get_authors(&self.0, pkg_ver_id).await
    }

    async fn get_package_url(
        &self,
        pkg: Package
    ) -> Result<String, AppError>
    {
        get_package_url(&self.0, pkg).await
    }

     async fn get_release_url(
        &self,
        pkg: Package,
        version: &Version
    ) -> Result<String, AppError>
    {
        get_release_url(&self.0, pkg, version).await
    }

    async fn get_players(
        &self,
        proj: Project
    ) -> Result<Users, AppError>
    {
        get_players(&self.0, proj).await
    }

    async fn add_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), AppError>
    {
        add_player(&self.0, player, proj).await
    }

    async fn remove_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), AppError>
    {
        remove_player(&self.0, player, proj).await
    }

    async fn get_image_url(
        &self,
        proj: Project,
        img_name: &str
    ) -> Result<String, AppError>
    {
        get_image_url(&self.0, proj, img_name).await
    }

    async fn get_image_url_at(
        &self,
        proj: Project,
        img_name: &str,
        date: i64
    ) -> Result<String, AppError>
    {
        get_image_url_at(&self.0, proj, img_name, date).await
    }

    async fn add_image_url(
        &self,
        owner: Owner,
        proj: Project,
        img_name: &str,
        url: &str,
        now: i64
    ) -> Result<(), AppError>
    {
        add_image_url(&self.0, owner, proj, img_name, url, now).await
    }
}

async fn get_project_id<'e, E>(
    ex: E,
    name: &str
) -> Result<Project, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT project_id
FROM projects
WHERE name = ?
        ",
        name
    )
    .fetch_optional(ex)
    .await?
    .map(Project)
    .ok_or(AppError::NotAProject)
}

async fn get_projects_count<'e, E>(
    ex: E
) -> Result<i64, AppError>
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
        .into()
    )
}

async fn get_projects_query_count<'e, E>(
    ex: E,
    query: &str
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT COUNT(1)
FROM projects_fts
WHERE projects_fts MATCH ?
            ",
            query
        )
        .fetch_one(ex)
        .await?
    )
}

async fn get_user_id<'e, E>(
    ex: E,
    username: &str
) -> Result<User, AppError>
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
        username
    )
    .fetch_optional(ex)
    .await?
    .map(User)
    .ok_or(AppError::NotAUser)
}

async fn get_owners<'e, E>(
    ex: E,
    proj: Project
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
                proj.0
            )
            .fetch_all(ex)
            .await?
        }
    )
}

async fn user_is_owner<'e, E>(
    ex: E,
    user: User,
    proj: Project
) -> Result<bool, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 AS present
FROM owners
WHERE user_id = ? AND project_id = ?
LIMIT 1
            ",
            user.0,
            proj.0
        )
        .fetch_optional(ex)
        .await?
        .is_some()
    )
}

async fn add_owner<'e, E>(
    ex: E,
    user: User,
    proj: Project
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
        user.0,
        proj.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn add_owners<'a, A>(
    conn: A,
    owners: &Users,
    proj: Project
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    for username in &owners.users {
        // get user id of new owner
        let owner = get_user_id(&mut *tx, username).await?;
        // associate new owner with the project
        add_owner(&mut *tx, owner, proj).await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn remove_owner<'e, E>(
    ex: E,
    user: User,
    proj: Project
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
        user.0,
        proj.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn remove_owners<'a, A>(
    conn: A,
    owners: &Users,
    proj: Project
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    for username in &owners.users {
        // get user id of owner
        let owner = get_user_id(&mut *tx, username).await?;
        // remove old owner from the project
        remove_owner(&mut *tx, owner, proj).await?;
    }

    // prevent removal of last owner
    if !has_owner(&mut *tx, proj).await? {
        return Err(AppError::CannotRemoveLastOwner);
    }

    tx.commit().await?;

    Ok(())
}

async fn has_owner<'e, E>(
    ex: E,
    proj: Project
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
            proj.0
        )
        .fetch_optional(ex)
        .await?
        .is_some()
    )
}

impl SortBy {
    fn field(&self) -> &'static str {
        match self {
            SortBy::ProjectName => "projects.name COLLATE NOCASE",
            SortBy::GameTitle => "projects.game_title_sort COLLATE NOCASE",
            SortBy::ModificationTime => "projects.modified_at",
            SortBy::CreationTime => "projects.created_at",
            // NB: "fts" is the table alias for the subquery
            SortBy::Relevance => "fts.rank"
        }
    }
}

impl Direction {
    fn dir(&self) -> &'static str {
        match self {
            Direction::Ascending => "ASC",
            Direction::Descending => "DESC"
        }
    }

    fn op(&self) -> &'static str {
        match self {
            Direction::Ascending => ">",
            Direction::Descending => "<"
        }
    }
}

async fn get_projects_end_window<'e, E>(
    ex: E,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        QueryBuilder::new(
            "
SELECT
    0.0 AS rank,
    project_id,
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year,
    image
FROM projects
ORDER BY "
        )
        .push(sort_by.field())
        .push(" ")
        .push(dir.dir())
        .push(", project_id ")
        .push(dir.dir())
        .push(" LIMIT ")
        .push_bind(limit)
        .build_query_as::<ProjectSummaryRow>()
        .fetch_all(ex)
        .await?
    )
}

async fn get_projects_query_end_window<'e, E>(
    ex: E,
    query: &str,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        QueryBuilder::new(
            "
SELECT
    fts.rank,
    projects.project_id,
    projects.name,
    projects.description,
    projects.revision,
    projects.created_at,
    projects.modified_at,
    projects.game_title,
    projects.game_title_sort,
    projects.game_publisher,
    projects.game_year,
    projects.image
FROM projects
JOIN projects_fts AS fts
ON projects.project_id = fts.rowid
WHERE projects_fts MATCH "
        )
        .push_bind(query)
        .push(" ORDER BY ")
        .push(sort_by.field())
        .push(" ")
        .push(dir.dir())
        .push(", projects.project_id ")
        .push(dir.dir())
        .push(" LIMIT ")
        .push_bind(limit)
        .build_query_as::<ProjectSummaryRow>()
        .fetch_all(ex)
        .await?
    )
}

async fn get_projects_mid_window<'e, 'f, E, F>(
    ex: E,
    sort_by: SortBy,
    dir: Direction,
    field: &'f F,
    id: u32,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>,
    F: Send + Sync + Encode<'f, Sqlite> + Type<Sqlite>
{
    Ok(
        QueryBuilder::new(
            "
SELECT
    0.0 AS rank,
    project_id,
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year,
    image
FROM projects
WHERE "
        )
        .push(sort_by.field())
        .push(" ")
        .push(dir.op())
        .push(" ")
        .push_bind(field)
        .push(" OR (")
        .push(sort_by.field())
        .push(" = ")
        .push_bind(field)
        .push(" AND project_id ")
        .push(dir.op())
        .push(" ")
        .push_bind(id)
        .push(") ORDER BY ")
        .push(sort_by.field())
        .push(" ")
        .push(dir.dir())
        .push(", project_id ")
        .push(dir.dir())
        .push(" LIMIT ")
        .push_bind(limit)
        .build_query_as::<ProjectSummaryRow>()
        .fetch_all(ex)
        .await?
    )
}

async fn get_projects_query_mid_window<'e, 'f, E, F>(
    ex: E,
    query: &'f str,
    sort_by: SortBy,
    dir: Direction,
    field: &'f F,
    id: u32,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>,
    F: Send + Sync + Encode<'f, Sqlite> + Type<Sqlite>
{
    // We get rows from the FTS table in a subquery because the sqlite
    // query planner is confused by MATCH when it's used with boolean
    // connectives.
    Ok(
        QueryBuilder::new(
            "
SELECT
    fts.rank,
    projects.project_id,
    projects.name,
    projects.description,
    projects.revision,
    projects.created_at,
    projects.modified_at,
    projects.game_title,
    projects.game_title_sort,
    projects.game_publisher,
    projects.game_year,
    projects.image
FROM projects
JOIN (
    SELECT
        projects_fts.rowid,
        projects_fts.rank
    FROM projects_fts
    WHERE projects_fts MATCH "
        )
        .push_bind(query)
        .push(") AS fts ON fts.rowid = projects.project_id WHERE ")
        .push(sort_by.field())
        .push(dir.op())
        .push(" ")
        .push_bind(field)
        .push(" OR (")
        .push(sort_by.field())
        .push(" = ")
        .push_bind(field)
        .push(" AND project_id ")
        .push(dir.op())
        .push(" ")
        .push_bind(id)
        .push(") ORDER BY ")
        .push(sort_by.field())
        .push(" ")
        .push(dir.dir())
        .push(", project_id ")
        .push(dir.dir())
        .push(" LIMIT ")
        .push_bind(limit)
        .build_query_as::<ProjectSummaryRow>()
        .fetch_all(ex)
        .await?
    )
}

async fn create_project_row<'e, E>(
    ex: E,
    user: User,
    proj: &str,
    proj_data: &ProjectDataPost,
    now: i64
) -> Result<Project, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        Project(
            sqlx::query_scalar!(
                "
INSERT INTO projects (
    name,
    created_at,
    description,
    game_title,
    game_title_sort,
    game_publisher,
    game_year,
    readme,
    image,
    modified_at,
    modified_by,
    revision
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
RETURNING project_id
                ",
                proj,
                now,
                proj_data.description,
                proj_data.game.title,
                proj_data.game.title_sort_key,
                proj_data.game.publisher,
                proj_data.game.year,
                "",
                None::<&str>,
                now,
                user.0,
                1
            )
            .fetch_one(ex)
            .await?
        )
    )
}

#[derive(Debug)]
struct ProjectDataRow<'a> {
    project_id: i64,
    description: &'a str,
    game_title: &'a str,
    game_title_sort: &'a str,
    game_publisher: &'a str,
    game_year: &'a str,
    readme: &'a str,
    image: Option<&'a str>
}

async fn create_project_data_row<'e, E>(
    ex: E,
    row: &ProjectDataRow<'_>
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
INSERT INTO project_data (
    project_id,
    description,
    game_title,
    game_title_sort,
    game_publisher,
    game_year,
    readme,
    image
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
RETURNING project_data_id
            ",
            row.project_id,
            row.description,
            row.game_title,
            row.game_title_sort,
            row.game_publisher,
            row.game_year,
            row.readme,
            row.image
        )
        .fetch_one(ex)
        .await?
    )
}

#[derive(Debug)]
struct ProjectRevisionRow<'a> {
    project_id: i64,
    name: &'a str,
    created_at: i64,
    modified_at: i64,
    modified_by: i64,
    revision: i64,
    project_data_id: i64
}

async fn create_project_revision_row<'e, E>(
    ex: E,
    row: &ProjectRevisionRow<'_>
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO project_revisions (
    project_id,
    name,
    created_at,
    modified_at,
    modified_by,
    revision,
    project_data_id
)
VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        row.project_id,
        row.name,
        row.created_at,
        row.modified_at,
        row.modified_by,
        row.revision,
        row.project_data_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn create_project<'a, A>(
    conn: A,
    owner: User,
    name: &str,
    pd: &ProjectDataPost,
    now: i64
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // create project row
    let proj = create_project_row(&mut *tx, owner, name, pd, now).await?;

    // associate new owner with the project
    add_owner(&mut *tx, owner, proj).await?;

    // create project revision
    let dr = ProjectDataRow {
        project_id: proj.0,
        description: &pd.description,
        game_title: &pd.game.title,
        game_title_sort: &pd.game.title_sort_key,
        game_publisher:  &pd.game.publisher,
        game_year: &pd.game.year,
        readme: &pd.readme,
        image: pd.image.as_deref()
    };

    let project_data_id = create_project_data_row(&mut *tx, &dr).await?;

    let rr = ProjectRevisionRow {
        project_id: proj.0,
        name,
        created_at: now,
        modified_at: now,
        modified_by: owner.0,
        revision: 1,
        project_data_id
    };

    create_project_revision_row(&mut *tx, &rr).await?;

    tx.commit().await?;

    Ok(())
}

async fn update_project_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    revision: i64,
    pd: &ProjectDataPatch,
    now: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut qb: QueryBuilder<E::Database> = QueryBuilder::new(
        "UPDATE projects SET "
    );

    let mut qbs = qb.separated(", ");

    qbs
        .push("revision = ")
        .push_bind_unseparated(revision)
        .push("modified_at = ")
        .push_bind_unseparated(now)
        .push("modified_by = ")
        .push_bind_unseparated(owner.0);

    if let Some(description) = &pd.description {
        qbs.push("description = ").push_bind_unseparated(description);
    }

    if let Some(game_title) = &pd.game.title {
        qbs.push("game_title = ").push_bind_unseparated(game_title);
    }

    if let Some(game_title_sort) = &pd.game.title_sort_key {
        qbs.push("game_title_sort = ").push_bind_unseparated(game_title_sort);
    }

    if let Some(game_publisher) = &pd.game.publisher {
        qbs.push("game_publisher = ").push_bind_unseparated(game_publisher);
    }

    if let Some(game_year) = &pd.game.year {
        qbs.push("game_year = ").push_bind_unseparated(game_year);
    }

    if let Some(readme) = &pd.readme {
        qbs.push("readme = ").push_bind_unseparated(readme);
    }

    if let Some(image) = &pd.image {
        qbs.push("image = ").push_bind_unseparated(image);
    }

    qb
        .push(" WHERE project_id = ")
        .push_bind(proj.0)
        .build()
        .execute(ex)
        .await?;

    Ok(())
}

// TODO: update project mtime when packages change

async fn update_project<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    pd: &ProjectDataPatch,
    now: i64
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // get project
    let row = get_project_row(&mut *tx, proj).await?;
    let revision = row.revision + 1;

    // update project
    update_project_row(&mut *tx, owner, proj, revision, pd, now).await?;

    // create project revision
    let dr = ProjectDataRow {
        project_id: proj.0,
        description: pd.description.as_ref().unwrap_or(&row.description),
        game_title: pd.game.title.as_ref().unwrap_or(&row.game_title),
        game_title_sort: pd.game.title_sort_key.as_ref().unwrap_or(&row.game_title_sort),
        game_publisher: pd.game.publisher.as_ref().unwrap_or(&row.game_publisher),
        game_year: pd.game.year.as_ref().unwrap_or(&row.game_year),
        readme: pd.readme.as_ref().unwrap_or(&row.readme),
        image: pd.image.as_ref().unwrap_or(&row.image).as_deref()
    };

    let project_data_id = create_project_data_row(&mut *tx, &dr).await?;

    let rr = ProjectRevisionRow {
        project_id: proj.0,
        name: &row.name,
        created_at: row.created_at,
        modified_at: now,
        modified_by: owner.0,
        revision,
        project_data_id
    };

    create_project_revision_row(&mut *tx, &rr).await?;

    tx.commit().await?;

    Ok(())
}

async fn get_project_row<'e, E>(
    ex: E,
    proj: Project
) -> Result<ProjectRow, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_as!(
        ProjectRow,
        "
SELECT
    project_id,
    name,
    description,
    revision,
    created_at,
    modified_at,
    modified_by,
    game_title,
    game_title_sort,
    game_publisher,
    game_year,
    readme,
    image
FROM projects
WHERE project_id = ?
LIMIT 1
        ",
        proj.0
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotAProject)
}

async fn get_project_row_revision<'e, E>(
    ex: E,
    proj: Project,
    revision: i64
) -> Result<ProjectRow, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_as!(
        ProjectRow,
        "
SELECT
    project_revisions.project_id,
    project_revisions.name,
    project_data.description,
    project_revisions.revision,
    project_revisions.created_at,
    project_revisions.modified_at,
    project_revisions.modified_by,
    project_data.game_title,
    project_data.game_title_sort,
    project_data.game_publisher,
    project_data.game_year,
    project_data.image,
    project_data.readme
FROM project_revisions
JOIN project_data
ON project_revisions.project_data_id = project_data.project_data_id
WHERE project_revisions.project_id = ?
    AND project_revisions.revision = ?
LIMIT 1
        ",
        proj.0,
        revision
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotARevision)
}

async fn get_packages<'e, E>(
    ex: E,
    proj: Project
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
    name,
    created_at
FROM packages
WHERE project_id = ?
ORDER BY name COLLATE NOCASE ASC
            ",
            proj.0
        )
       .fetch_all(ex)
       .await?
    )
}

async fn get_packages_at<'e, E>(
    ex: E,
    proj: Project,
    date: i64
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
    name,
    created_at
FROM packages
WHERE project_id = ?
    AND created_at <= ?
ORDER BY name COLLATE NOCASE ASC
            ",
            proj.0,
            date
        )
       .fetch_all(ex)
       .await?
    )
}

async fn create_package<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    pkg: &str,
    pkg_data: &PackageDataPost,
    now: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO packages (
    project_id,
    name,
    created_at,
    created_by
)
VALUES (?, ?, ?, ?)
            ",
            proj.0,
            pkg,
            now,
            owner.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

// TODO: can we combine these?
// TODO: make Version borrow Strings?
impl<'r> From<&'r ReleaseRow> for Version {
    fn from(r: &'r ReleaseRow) -> Self {
        Version {
            major: r.version_major,
            minor: r.version_minor,
            patch: r.version_patch,
            pre: Some(&r.version_pre).filter(|v| !v.is_empty()).cloned(),
            build: Some(&r.version_build).filter(|v| !v.is_empty()).cloned()
        }
    }
}

impl<'r> From<&'r ReducedReleaseRow> for Version {
    fn from(r: &'r ReducedReleaseRow) -> Self {
        Version {
            major: r.version_major,
            minor: r.version_minor,
            patch: r.version_patch,
            pre: Some(&r.version_pre).filter(|v| !v.is_empty()).cloned(),
            build: Some(&r.version_build).filter(|v| !v.is_empty()).cloned()
        }
    }
}

fn release_row_cmp<R>(a: &R, b: &R) -> Ordering
where
    Version: for<'r> From<&'r R>
{
    let av: Version = a.into();
    let bv = b.into();
    av.cmp(&bv)
}

async fn get_releases<'e, E>(
    ex: E,
    pkg: Package
) -> Result<Vec<ReleaseRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReleaseRow,
        "
SELECT
    releases.release_id,
    releases.version,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build,
    releases.url,
    releases.filename,
    releases.size,
    releases.checksum,
    releases.published_at,
    users.username AS published_by
FROM releases
JOIN users
ON releases.published_by = users.user_id
WHERE package_id = ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC,
    version_pre ASC,
    version_build ASC
        ",
        pkg.0
    )
    .fetch_all(ex)
    .await?;

    releases.sort_by(|a, b| release_row_cmp(b, a));
    Ok(releases)
}

async fn get_releases_at<'e, E>(
    ex: E,
    pkg: Package,
    date: i64
) -> Result<Vec<ReleaseRow>, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReleaseRow,
        "
SELECT
    releases.release_id,
    releases.version,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build,
    releases.url,
    releases.filename,
    releases.size,
    releases.checksum,
    releases.published_at,
    users.username AS published_by
FROM releases
JOIN users
ON releases.published_by = users.user_id
WHERE package_id = ?
    AND published_at <= ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC,
    version_pre ASC,
    version_build ASC
        ",
        pkg.0,
        date
    )
    .fetch_all(ex)
    .await?;

    releases.sort_by(|a, b| release_row_cmp(b, a));
    Ok(releases)
}

async fn get_authors<'e, E>(
    ex: E,
    pkg_ver_id: i64
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

#[derive(Debug, Deserialize)]
struct ReducedReleaseRow {
    url: String,
    version_major: i64,
    version_minor: i64,
    version_patch: i64,
    version_pre: String,
    version_build: String,
}

// TODO: figure out how to order version_pre
async fn get_package_url<'e, E>(
    ex: E,
    pkg: Package
) -> Result<String, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReducedReleaseRow,
        "
SELECT
    url,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build
FROM releases
WHERE package_id = ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC,
    version_pre ASC,
    version_build ASC
        ",
        pkg.0
    )
    .fetch_all(ex)
    .await?;

    match releases.is_empty() {
        true => Err(AppError::NotAPackage),
        false => {
            releases.sort_by(release_row_cmp);
            Ok(releases.pop().unwrap().url)
        }
    }
}

async fn get_release_url<'e, E>(
    ex: E,
    pkg: Package,
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
FROM releases
WHERE package_id = ?
    AND version_major = ?
    AND version_minor = ?
    AND version_patch = ?
    AND version_pre = ?
    AND version_build = ?
LIMIT 1
        ",
        pkg.0,
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
    proj: Project
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
                proj.0
            )
            .fetch_all(ex)
            .await?
        }
    )
}

async fn add_player<'e, E>(
    ex: E,
    user: User,
    proj: Project
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
        user.0,
        proj.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn remove_player<'e, E>(
    ex: E,
    user: User,
    proj: Project
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
        user.0,
        proj.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn get_image_url<'e, E>(
    ex: E,
    proj: Project,
    img_name: &str
) -> Result<String, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT url
FROM images
WHERE project_id = ?
    AND filename = ?
LIMIT 1
        ",
        proj.0,
        img_name
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotFound)
}

// TODO: tests
async fn get_image_url_at<'e, E>(
    ex: E,
    proj: Project,
    img_name: &str,
    date: i64
) -> Result<String, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT url
FROM image_revisions
WHERE project_id = ?
    AND filename = ?
    AND published_at <= ?
ORDER BY published_at DESC
LIMIT 1
        ",
        proj.0,
        img_name,
        date
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotFound)
}

// TODO: tests
async fn update_image_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO images (
    project_id,
    filename,
    url,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?)
ON CONFLICT(project_id, filename)
DO UPDATE
SET url = excluded.url,
    published_at = excluded.published_at,
    published_by = excluded.published_by
        ",
        proj.0,
        img_name,
        url,
        now,
        owner.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

// TODO: tests
async fn create_image_revision_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO image_revisions (
    project_id,
    filename,
    url,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?)
        ",
        proj.0,
        img_name,
        url,
        now,
        owner.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

// TODO: tests
async fn get_project_data_id<'e, E>(
    ex: E,
    proj: Project,
    revision: i64
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT
    project_revisions.project_data_id
FROM project_revisions
WHERE project_revisions.project_id = ?
    AND project_revisions.revision = ?
LIMIT 1
        ",
        proj.0,
        revision
    )
    .fetch_optional(ex)
    .await?
    .ok_or(AppError::NotARevision)
}

// TODO: tests
async fn add_image_url<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64,
) -> Result<(), AppError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // update row in images
    update_image_row(
        &mut *tx,
        owner,
        proj,
        img_name,
        url,
        now
    ).await?;

    // insert row in images_revisions
    create_image_revision_row(
        &mut *tx,
        owner,
        proj,
        img_name,
        url,
        now
    ).await?;

    // get the project row, project_data_id
    let row = get_project_row(&mut *tx, proj).await?;

    let project_data_id = get_project_data_id(&mut *tx, proj, row.revision)
        .await?;

    let revision = row.revision + 1;

    // insert a new project revision row
    let rr = ProjectRevisionRow {
        project_id: proj.0,
        name: &row.name,
        created_at: row.created_at,
        modified_at: now,
        modified_by: owner.0,
        revision,
        project_data_id
    };

    create_project_revision_row(&mut *tx, &rr).await?;

    // update the project row
    update_project_row(
        &mut *tx,
        owner,
        proj,
        revision,
        &Default::default(),
        now
    ).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::GameData;

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_id_ok(pool: Pool) {
        assert_eq!(
            get_project_id(&pool, "test_game").await.unwrap(),
            Project(42)
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_id_not_a_project(pool: Pool) {
        assert_eq!(
            get_project_id(&pool, "bogus").await.unwrap_err(),
            AppError::NotAProject
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_count_ok(pool: Pool) {
        assert_eq!(get_projects_count(&pool).await.unwrap(), 2);
    }

    #[sqlx::test(fixtures("users"))]
    async fn get_user_id_ok(pool: Pool) {
        assert_eq!(get_user_id(&pool, "bob").await.unwrap(), User(1));
    }

    #[sqlx::test(fixtures("users"))]
    async fn get_user_id_not_a_user(pool: Pool) {
        assert_eq!(
            get_user_id(&pool, "not_a_user").await.unwrap_err(),
            AppError::NotAUser
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
    }

// TODO: can we tell when the project doesn't exist?
/*
    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn get_owners_not_a_project(pool: Pool) {
        assert_eq!(
            get_owners(&pool, 0).await.unwrap_err(),
            AppError::NotAProject
        );
    }
*/

// TODO: can we tell when the project doesn't exist?

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        assert!(user_is_owner(&pool, User(1), Project(42)).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects","one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        assert!(!user_is_owner(&pool, User(2), Project(42)).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn add_owner_new(pool: Pool) {
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
        add_owner(&pool, User(2), Project(42)).await.unwrap();
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into()
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn add_owner_existing(pool: Pool) {
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
        add_owner(&pool, User(1), Project(42)).await.unwrap();
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
    }

// TODO: add test for add_owner not a project
// TODO: add test for add_owner not a user

    #[sqlx::test(fixtures("users", "projects","one_owner"))]
    async fn remove_owner_ok(pool: Pool) {
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
        remove_owner(&pool, User(1), Project(42)).await.unwrap();
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec![] }
        );
    }

    #[sqlx::test(fixtures( "users", "projects", "one_owner"))]
    async fn remove_owner_not_an_owner(pool: Pool) {
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
        remove_owner(&pool, User(2), Project(42)).await.unwrap();
        assert_eq!(
            get_owners(&pool, Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
    }

// TODO: add test for remove_owner not a project

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn has_owner_yes(pool: Pool) {
        assert!(has_owner(&pool, Project(42)).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn has_owner_no(pool: Pool) {
        assert!(!has_owner(&pool, Project(42)).await.unwrap());
    }

    #[sqlx::test(fixtures("users"))]
    async fn create_project_ok(pool: Pool) {
        let user = User(1);
        let row = ProjectRow {
            project_id: 1,
            name: "test_game".into(),
            description: "Brian's Trademarked Game of Being a Test Case".into(),
            revision: 1,
            created_at: 1699804206419538067,
            modified_at: 1699804206419538067,
            modified_by: 1,
            game_title: "A Game of Tests".into(),
            game_title_sort: "Game of Tests, A".into(),
            game_publisher: "Test Game Company".into(),
            game_year: "1979".into(),
            readme: "".into(),
            image: None
        };

        let cdata = ProjectDataPost {
            description: row.description.clone(),
            tags: vec![],
            game: GameData {
                title: row.game_title.clone(),
                title_sort_key: row.game_title_sort.clone(),
                publisher: row.game_publisher.clone(),
                year: row.game_year.clone()
            },
            readme: "".into(),
            image: None
        };

        assert_eq!(
            get_project_id(&pool, &row.name).await.unwrap_err(),
            AppError::NotAProject
        );

        create_project(
            &pool,
            user,
            &row.name,
            &cdata,
            row.created_at
        ).await.unwrap();

        let proj = get_project_id(&pool, &row.name).await.unwrap();

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap(),
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

    #[track_caller]
    fn assert_projects_window(
        act: Result<Vec<ProjectSummaryRow>, AppError>,
        exp: &[&str]
    )
    {
        assert_eq!(
            act.unwrap().into_iter().map(|r| r.name).collect::<Vec<_>>(),
            exp
        );
    }

    #[sqlx::test]
    async fn get_projects_end_window_asc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_end_window(
                &pool, SortBy::ProjectName, Direction::Ascending, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_end_window_asc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_end_window(
                &pool, SortBy::ProjectName, Direction::Ascending, 3
            ).await,
            &["a", "b", "c"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_end_window_asc_past_end(pool: Pool) {
        assert_projects_window(
            get_projects_end_window(
                &pool, SortBy::ProjectName, Direction::Ascending, 5
            ).await,
            &["a", "b", "c", "d"]
        );
    }

    #[sqlx::test]
    async fn get_projects_end_window_desc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_end_window(
                &pool, SortBy::ProjectName, Direction::Descending, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_end_window_desc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_end_window(
                &pool, SortBy::ProjectName, Direction::Descending, 3
            ).await,
            &["d", "c", "b"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_end_window_desc_past_start(pool: Pool) {
        assert_projects_window(
            get_projects_end_window(
                &pool, SortBy::ProjectName, Direction::Descending, 5
            ).await,
            &["d", "c", "b", "a"]
        );
    }

    #[sqlx::test]
    async fn get_projects_mid_window_asc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_mid_window(
                &pool, SortBy::ProjectName, Direction::Ascending, &"a", 1, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_mid_window_asc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_mid_window(
                &pool, SortBy::ProjectName, Direction::Ascending, &"b", 2, 3
            ).await,
            &["c", "d"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_mid_window_asc_past_end(pool: Pool) {
        assert_projects_window(
            get_projects_mid_window(
                &pool, SortBy::ProjectName, Direction::Ascending, &"d", 4, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test]
    async fn get_projects_mid_window_desc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_mid_window(
                &pool, SortBy::ProjectName, Direction::Descending, &"a", 1, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_mid_window_desc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_mid_window(
                &pool, SortBy::ProjectName, Direction::Descending, &"b", 2, 3
            ).await,
            &["a"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_window"))]
    async fn get_projects_mid_window_desc_past_start(pool: Pool) {
        assert_projects_window(
            get_projects_mid_window(
                &pool, SortBy::ProjectName, Direction::Descending, &"d", 4, 3
            ).await,
            &["c", "b", "a"]
        );
    }

    #[sqlx::test]
    async fn get_projects_query_end_window_asc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_query_end_window(
                &pool, "abc", SortBy::ProjectName, Direction::Ascending, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_end_window_asc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_query_end_window(
                &pool, "abc", SortBy::ProjectName, Direction::Ascending, 1
            ).await,
            &["a"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_end_window_asc_past_end(pool: Pool) {
        assert_projects_window(
            get_projects_query_end_window(
                &pool, "abc", SortBy::ProjectName, Direction::Ascending, 5
            ).await,
            &["a", "c", "d"]
        );
    }

    #[sqlx::test]
    async fn get_projects_query_end_window_desc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_query_end_window(
                &pool, "abc", SortBy::ProjectName, Direction::Descending, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_end_window_desc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_query_end_window(
                &pool, "abc", SortBy::ProjectName, Direction::Descending, 1
            ).await,
            &["d"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_end_window_desc_past_start(pool: Pool) {
        assert_projects_window(
            get_projects_query_end_window(
                &pool, "abc", SortBy::ProjectName, Direction::Descending, 5
            ).await,
            &["d", "c", "a"]
        );
    }

    #[sqlx::test]
    async fn get_projects_query_mid_window_asc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_query_mid_window(
                &pool, "abc", SortBy::ProjectName, Direction::Ascending, &"a", 1, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_mid_window_asc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_query_mid_window(
                &pool, "abc", SortBy::ProjectName, Direction::Ascending, &"b", 2, 3
            ).await,
            &["c", "d"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_mid_window_asc_past_end(pool: Pool) {
        assert_projects_window(
            get_projects_query_mid_window(
                &pool, "abc", SortBy::ProjectName, Direction::Ascending, &"d", 4, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test]
    async fn get_projects_query_mid_window_desc_empty(pool: Pool) {
        assert_projects_window(
            get_projects_query_mid_window(
                &pool, "abc", SortBy::ProjectName, Direction::Descending, &"a", 1, 3
            ).await,
            &[]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_mid_window_desc_not_all(pool: Pool) {
        assert_projects_window(
            get_projects_query_mid_window(
                &pool, "abc", SortBy::ProjectName, Direction::Descending, &"d", 4, 1
            ).await,
            &["c"]
        );
    }

    #[sqlx::test(fixtures("users", "proj_query_window"))]
    async fn get_projects_query_mid_window_desc_past_start(pool: Pool) {
        assert_projects_window(
            get_projects_query_mid_window(
                &pool, "abc", SortBy::ProjectName, Direction::Descending, &"d", 4, 5
            ).await,
            &["c", "a"]
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_row_ok(pool: Pool) {
        assert_eq!(
            get_project_row(&pool, Project(42)).await.unwrap(),
            ProjectRow {
                project_id: 42,
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 3,
                created_at: 1699804206419538067,
                modified_at: 1702569006419538067,
                modified_by: 1,
                game_title: "A Game of Tests".into(),
                game_title_sort: "Game of Tests, A".into(),
                game_publisher: "Test Game Company".into(),
                game_year: "1979".into(),
                readme: "".into(),
                image: None
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_row_not_a_project(pool: Pool) {
        assert_eq!(
            get_project_row(&pool, Project(0)).await.unwrap_err(),
            AppError::NotAProject
        );
    }

    #[sqlx::test(fixtures("users", "projects", "two_owners"))]
    async fn get_project_row_revision_ok_current(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, Project(42), 3).await.unwrap(),
            ProjectRow {
                project_id: 42,
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 3,
                created_at: 1699804206419538067,
                modified_at: 1702569006419538067,
                modified_by: 1,
                game_title: "A Game of Tests".into(),
                game_title_sort: "Game of Tests, A".into(),
                game_publisher: "Test Game Company".into(),
                game_year: "1979".into(),
                readme: "".into(),
                image: None
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "two_owners"))]
    async fn get_project_revision_ok_old(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, Project(42), 1).await.unwrap(),
            ProjectRow {
                project_id: 42,
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 1,
                created_at: 1699804206419538067,
                modified_at: 1699804206419538067,
                modified_by: 1,
                game_title: "A Game of Tests".into(),
                game_title_sort: "Game of Tests, A".into(),
                game_publisher: "Test Game Company".into(),
                game_year: "1978".into(),
                readme: "".into(),
                image: None
            }
        );
    }

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_revision_not_a_project(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, Project(0), 2).await.unwrap_err(),
            AppError::NotARevision
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_revision_not_a_revision(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, Project(42), 0).await.unwrap_err(),
            AppError::NotARevision
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_ok(pool: Pool) {
        assert_eq!(
            get_packages(&pool, Project(42)).await.unwrap(),
            vec![
                PackageRow {
                    package_id: 1,
                    name: "a_package".into(),
                    created_at: 1702137389180282477
                },
                PackageRow {
                    package_id: 2,
                    name: "b_package".into(),
                    created_at: 1667750189180282477
                },
                PackageRow {
                    package_id: 3,
                    name: "c_package".into(),
                    created_at: 1699286189180282477
                }
            ]
        );
    }

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_not_a_project(pool: Pool) {
        assert_eq!(
            get_packages(&pool, Project(0)).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_at_none(pool: Pool) {
        let date = 0;
        assert_eq!(
            get_packages_at(&pool, Project(42), date).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_at_some(pool: Pool) {
        let date = 1672531200000000000;
        assert_eq!(
            get_packages_at(&pool, Project(42), date).await.unwrap(),
            vec![
                PackageRow {
                    package_id: 2,
                    name: "b_package".into(),
                    created_at: 1667750189180282477
                }
            ]
        );
    }

    // TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_at_not_a_project(pool: Pool) {
        let date = 16409952000000000;
        assert_eq!(
            get_packages_at(&pool, Project(0), date).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_package_url_ok(pool: Pool) {
        assert_eq!(
            get_package_url(&pool, Package(1)).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_package_url_not_a_package(pool: Pool) {
        assert_eq!(
            get_package_url(&pool, Package(0)).await.unwrap_err(),
            AppError::NotAPackage
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn create_package_ok(pool: Pool) {
        assert_eq!(
            get_packages(&pool, Project(6)).await.unwrap(),
            []
        );

        create_package(
            &pool,
            Owner(1),
            Project(6),
            "newpkg",
            &PackageDataPost {
                description: "".into()
            },
            1699804206419538067
        ).await.unwrap();

        assert_eq!(
            get_packages(&pool, Project(6)).await.unwrap(),
            [
                PackageRow {
                    package_id: 4,
                    name: "newpkg".into(),
                    created_at: 1699804206419538067
                }
            ]
        );
    }

// TODO: test create_package not a project
// TODO: test create_package duplicate name

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_ok(pool: Pool) {
        assert_eq!(
            get_releases(&pool, Package(1)).await.unwrap(),
            vec![
                ReleaseRow {
                    release_id: 2,
                    version: "1.2.4".into(),
                    version_major: 1,
                    version_minor: 2,
                    version_patch: 4,
                    version_pre: "".into(),
                    version_build: "".into(),
                    url: "https://example.com/a_package-1.2.4".into(),
                    filename: "a_package-1.2.4".into(),
                    size: 5678,
                    checksum: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
                    published_at: 1702223789180282477,
                    published_by: "alice".into()
                },
                ReleaseRow {
                    release_id: 1,
                    version: "1.2.3".into(),
                    version_major: 1,
                    version_minor: 2,
                    version_patch: 3,
                    version_pre: "".into(),
                    version_build: "".into(),
                    url: "https://example.com/a_package-1.2.3".into(),
                    filename: "a_package-1.2.3".into(),
                    size: 1234,
                    checksum: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
                    published_at: 1702137389180282477,
                    published_by: "bob".into()
                }
            ]
        );
    }

// TODO: can we tell when the package doesn't exist?
    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_not_a_package(pool: Pool) {
        assert_eq!(
            get_releases(&pool, Package(0)).await.unwrap(),
            vec![]
        );
    }

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

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn get_players_ok(pool: Pool) {
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into()
                ]
            }
        );
    }

// TODO: can we tell when the project doesn't exist?
    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn get_players_not_a_project(pool: Pool) {
        assert_eq!(
            get_players(&pool, Project(0)).await.unwrap(),
            Users { users: vec![] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn add_player_new(pool: Pool) {
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                ]
            }
        );
        add_player(&pool, User(3), Project(42)).await.unwrap();
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                    "chuck".into()
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn add_player_existing(pool: Pool) {
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                ]
            }
        );
        add_player(&pool, User(2), Project(42)).await.unwrap();
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                ]
            }
        );
    }

// TODO: add test for add_player not a project
// TODO: add test for add_player not a user

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn remove_player_existing(pool: Pool) {
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                ]
            }
        );
        remove_player(&pool, User(2), Project(42)).await.unwrap();
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "bob".into(),
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn remove_player_not_a_player(pool: Pool) {
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                ]
            }
        );
        remove_player(&pool, User(3), Project(42)).await.unwrap();
        assert_eq!(
            get_players(&pool, Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                ]
            }
        );
    }

// TODO: add test for remove_player not a project

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_ok(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "img.png").await.unwrap(),
            "https://example.com/images/img.png"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_not_a_project(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(1), "img.png").await.unwrap_err(),
            AppError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_not_an_image(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "bogus").await.unwrap_err(),
            AppError::NotFound
        );
    }
}
