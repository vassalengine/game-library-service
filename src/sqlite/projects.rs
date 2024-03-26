use sqlx::{
    Acquire, Encode, Executor, QueryBuilder, Transaction, Type,
    sqlite::Sqlite
};

use crate::{
    core::CoreError,
    db::{ProjectRow, ProjectSummaryRow},
    model::{Owner, Project, ProjectDataPatch, ProjectDataPost, User},
    pagination::{Direction, SortBy},
    sqlite::users::add_owner
};

pub async fn get_project_id<'e, E>(
    ex: E,
    name: &str
) -> Result<Project, CoreError>
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
    .ok_or(CoreError::NotAProject)
}

pub async fn get_projects_count<'e, E>(
    ex: E
) -> Result<i64, CoreError>
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

pub async fn get_projects_query_count<'e, E>(
    ex: E,
    query: &str
) -> Result<i64, CoreError>
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

// TODO: can we simplify cursors by using a subquery to get the value?

pub async fn get_projects_end_window<'e, E>(
    ex: E,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, CoreError>
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

pub async fn get_projects_query_end_window<'e, E>(
    ex: E,
    query: &str,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, CoreError>
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

pub async fn get_projects_mid_window<'e, 'f, E, F>(
    ex: E,
    sort_by: SortBy,
    dir: Direction,
    field: &'f F,
    id: u32,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, CoreError>
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

pub async fn get_projects_query_mid_window<'e, 'f, E, F>(
    ex: E,
    query: &'f str,
    sort_by: SortBy,
    dir: Direction,
    field: &'f F,
    id: u32,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, CoreError>
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

pub async fn create_project_row<'e, E>(
    ex: E,
    user: User,
    proj: &str,
    proj_data: &ProjectDataPost,
    now: i64
) -> Result<Project, CoreError>
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
pub struct ProjectDataRow<'a> {
    project_id: i64,
    description: &'a str,
    game_title: &'a str,
    game_title_sort: &'a str,
    game_publisher: &'a str,
    game_year: &'a str,
    readme: &'a str,
    image: Option<&'a str>
}

pub async fn create_project_data_row<'e, E>(
    ex: E,
    row: &ProjectDataRow<'_>
) -> Result<i64, CoreError>
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
pub struct ProjectRevisionRow<'a> {
    project_id: i64,
    name: &'a str,
    created_at: i64,
    modified_at: i64,
    modified_by: i64,
    revision: i64,
    project_data_id: i64
}

pub async fn create_project_revision_row<'e, E>(
    ex: E,
    row: &ProjectRevisionRow<'_>
) -> Result<(), CoreError>
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

pub async fn create_project<'a, A>(
    conn: A,
    owner: User,
    name: &str,
    pd: &ProjectDataPost,
    now: i64
) -> Result<(), CoreError>
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

pub async fn update_project_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    revision: i64,
    pd: &ProjectDataPatch,
    now: i64
) -> Result<(), CoreError>
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

pub async fn update_project<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    pd: &ProjectDataPatch,
    now: i64
) -> Result<(), CoreError>
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

pub async fn get_project_row<'e, E>(
    ex: E,
    proj: Project
) -> Result<ProjectRow, CoreError>
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
    .ok_or(CoreError::NotAProject)
}

pub async fn get_project_row_revision<'e, E>(
    ex: E,
    proj: Project,
    revision: i64
) -> Result<ProjectRow, CoreError>
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
    .ok_or(CoreError::NotARevision)
}

// TODO: tests
pub async fn get_project_data_id<'e, E>(
    ex: E,
    proj: Project,
    revision: i64
) -> Result<i64, CoreError>
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
    .ok_or(CoreError::NotARevision)
}

pub async fn update_project_non_project_data(
    tx: &mut Transaction<'_, Sqlite>,
    owner: Owner,
    proj: Project,
    now: i64,
) -> Result<(), CoreError>
{
    // get the project row, project_data_id
    let row = get_project_row(&mut **tx, proj).await?;

    let project_data_id = get_project_data_id(&mut **tx, proj, row.revision)
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

    create_project_revision_row(&mut **tx, &rr).await?;

    // update the project row
    update_project_row(
        &mut **tx,
        owner,
        proj,
        revision,
        &Default::default(),
        now
    ).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::model::GameData;

    type Pool = sqlx::Pool<Sqlite>;

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
            CoreError::NotAProject
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_count_ok(pool: Pool) {
        assert_eq!(get_projects_count(&pool).await.unwrap(), 2);
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
            CoreError::NotAProject
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
        act: Result<Vec<ProjectSummaryRow>, CoreError>,
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
            CoreError::NotAProject
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

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_revision_not_a_project(pool: Pool) {
        // This should not happen; the Project passed in should be good.
        assert_eq!(
            get_project_row_revision(&pool, Project(0), 2).await.unwrap_err(),
            CoreError::NotARevision
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_project_revision_not_a_revision(pool: Pool) {
        assert_eq!(
            get_project_row_revision(&pool, Project(42), 0).await.unwrap_err(),
            CoreError::NotARevision
        );
    }
}
