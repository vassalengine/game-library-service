use sqlx::{
    Encode, Executor, QueryBuilder, Type,
    sqlite::Sqlite
};

use crate::{
    db::{DatabaseError, ProjectSummaryRow},
    pagination::{Direction, SortBy}
};

pub async fn get_projects_count<'e, E>(
    ex: E
) -> Result<i64, DatabaseError>
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

pub async fn get_projects_query_count<'e, E>(
    ex: E,
    query: &str
) -> Result<i64, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let query = fts5_quote(query);
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

pub async fn get_projects_end_window<'e, E>(
    ex: E,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, DatabaseError>
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
    game_players_min,
    game_players_max,
    game_length_min,
    game_length_max,
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

fn fts5_quote(s: &str) -> String {
    // Wrapping a query in double quotes ensures that it is interpreted as
    // a string in the FTS5 query langauge, but then internal double quotes
    // must also be escaped with an extra double quote.
    format!("\"{}\"", s.replace("\"", "\"\""))
}

pub async fn get_projects_query_end_window<'e, E>(
    ex: E,
    query: &str,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, DatabaseError>
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
    projects.game_players_min,
    projects.game_players_max,
    projects.game_length_min,
    projects.game_length_max,
    projects.image
FROM projects
JOIN projects_fts AS fts
ON projects.project_id = fts.rowid
WHERE projects_fts MATCH "
        )
        .push_bind(fts5_quote(query))
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
) -> Result<Vec<ProjectSummaryRow>, DatabaseError>
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
    game_players_min,
    game_players_max,
    game_length_min,
    game_length_max,
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
) -> Result<Vec<ProjectSummaryRow>, DatabaseError>
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
    projects.game_players_min,
    projects.game_players_max,
    projects.game_length_min,
    projects.game_length_max,
    projects.image
FROM projects
JOIN (
    SELECT
        projects_fts.rowid,
        projects_fts.rank
    FROM projects_fts
    WHERE projects_fts MATCH "
        )
        .push_bind(fts5_quote(query))
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

#[cfg(test)]
mod test {
    use super::*;

    type Pool = sqlx::Pool<Sqlite>;

    async fn fts5_quote_abc() {
        assert_eq!("\"abc\"", fts5_quote("abc"));
    }

    async fn fts5_quote_abc_def() {
        assert_eq!("\"abc def\"", fts5_quote("abc def"));
    }

    async fn fts5_quote_abcq_def() {
        assert_eq!("\"abc\"\" def\"", fts5_quote("abc\" def"));
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_count_ok(pool: Pool) {
        assert_eq!(get_projects_count(&pool).await.unwrap(), 2);
    }

    #[track_caller]
    fn assert_projects_window(
        act: Result<Vec<ProjectSummaryRow>, DatabaseError>,
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
}
