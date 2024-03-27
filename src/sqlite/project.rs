use sqlx::{
    Acquire, Executor, QueryBuilder, Transaction,
    sqlite::Sqlite
};

use crate::{
    core::CoreError,
    db::ProjectRow,
    model::{Owner, Project, ProjectDataPatch, ProjectDataPost, User},
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

async fn create_project_row<'e, E>(
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

async fn update_project_row<'e, E>(
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
async fn get_project_data_id<'e, E>(
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

    use once_cell::sync::Lazy;

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

    static CREATE_ROW: Lazy<ProjectRow> = Lazy::new(|| {
        ProjectRow {
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
        }
    });

    static CREATE_DATA: Lazy<ProjectDataPost> = Lazy::new(|| {
        ProjectDataPost {
            description: CREATE_ROW.description.clone(),
            tags: vec![],
            game: GameData {
                title: CREATE_ROW.game_title.clone(),
                title_sort_key: CREATE_ROW.game_title_sort.clone(),
                publisher: CREATE_ROW.game_publisher.clone(),
                year: CREATE_ROW.game_year.clone()
            },
            readme: "".into(),
            image: None
        }
    });

    #[sqlx::test(fixtures("users"))]
    async fn create_project_ok(pool: Pool) {
        assert_eq!(
            get_project_id(&pool, &CREATE_ROW.name).await.unwrap_err(),
            CoreError::NotAProject
        );

        create_project(
            &pool,
            User(1),
            &CREATE_ROW.name,
            &CREATE_DATA,
            CREATE_ROW.created_at
        ).await.unwrap();

        let proj = get_project_id(&pool, &CREATE_ROW.name).await.unwrap();

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap(),
            *CREATE_ROW
        );
    }

    #[sqlx::test(fixtures("users"))]
    async fn create_project_not_a_user(pool: Pool) {
        assert_eq!(
            get_project_id(&pool, &CREATE_ROW.name).await.unwrap_err(),
            CoreError::NotAProject
        );

        assert!(
            matches!(
                create_project(
                    &pool,
                    User(0),
                    &CREATE_ROW.name,
                    &CREATE_DATA,
                    CREATE_ROW.created_at
                ).await.unwrap_err(),
                CoreError::DatabaseError(_)
            )
        );

        assert_eq!(
            get_project_id(&pool, &CREATE_ROW.name).await.unwrap_err(),
            CoreError::NotAProject
        );
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn create_project_already_exists(pool: Pool) {
        let row = ProjectRow {
            project_id: 42,
            ..CREATE_ROW.clone()
        };

        assert_eq!(
            get_project_id(&pool, &row.name).await.unwrap(),
            Project(row.project_id)
        );

        assert!(
            matches!(
                create_project(
                    &pool,
                    User(1),
                    &row.name,
                    &CREATE_DATA,
                    row.created_at
                ).await.unwrap_err(),
                CoreError::DatabaseError(_)
            )
        );
    }

// TODO: add tests for copy_project_revsion
// TODO: add tests for update_project

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
