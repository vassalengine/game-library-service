use const_format::formatcp;
use sqlx::{
    Encode, Executor, QueryBuilder, Type,
    query_builder::Separated,
    sqlite::Sqlite
};
use std::fmt;

use crate::{
    db::{DatabaseError, Facet, ProjectSummaryRow},
    pagination::{Direction, SortBy}
};

// TODO: put a QueryBuilder into the db object for reuse?

/*
impl fmt::Display for WhereValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WhereValue::Text(s) => write!(f, "{}", s),
            WhereValue::Integer(i) => write!(f, "{}", i)
        }
    }
}
*/

trait JoinExt {
    fn push_join(&mut self, wf: &Facet) -> &mut Self;
}

impl<'args> JoinExt for QueryBuilder<'args, Sqlite> {
    fn push_join(&mut self, wf: &Facet) -> &mut Self {
        match wf {
            Facet::Tag(_) => self.push(" JOIN tags ON projects.project_id = tags.project_id "),
            Facet::Owner(_) => self.push(" JOIN owners ON projects.project_id = owners.project_id JOIN users ON owners.user_id = users.user_id "),
            Facet::Player(_) => self.push(" JOIN players ON projects.project_id = players.project_id JOIN users ON owners.user_id = users.user_id "),
            _ => self
        }
    }
}

trait WhereExt<'args> {
    fn push_where(&mut self, wf: &'args Facet) -> &mut Self;
}

impl<'qb, 'args, Sep> WhereExt<'args> for Separated<'qb, 'args, Sqlite, Sep>
where
    Sep: std::fmt::Display
{
    fn push_where(&mut self, wf: &'args Facet) -> &mut Self
    {
        match wf {
            Facet::Publisher(p) =>
                self.push(" projects.game_publisher == ")
                    .push_bind_unseparated(p),
            Facet::Year(y) =>
                self.push(" projects.game_year == ")
                    .push_bind_unseparated(y),
            Facet::PlayersMin(m) =>
                self.push(" projects.game_players_min == ")
                    .push_bind_unseparated(m),
            Facet::PlayersMax(m) =>
                self.push(" projects.game_players_max == ")
                    .push_bind_unseparated(m),
            Facet::LengthMin(m) =>
                self.push(" projects.game_length_min == ")
                    .push_bind_unseparated(m),
            Facet::LengthMax(m) =>
                self.push(" projects.game_length_max == ")
                    .push_bind_unseparated(m),
            Facet::Tag(t) =>
                self.push(" tags.tag == ")
                    .push_bind_unseparated(t),
            Facet::Owner(u) |
            Facet::Player(u) =>
                self.push(" users.username == ")
                    .push_bind_unseparated(u)
        }
    }
}

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

/*
pub async fn get_projects_tag_count<'e, E>(
    ex: E,
    tag: &str
) -> Result<i64, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT COUNT(1)
FROM tags
WHERE tag = ?
            ",
            tag
        )
        .fetch_one(ex)
        .await?
    )
}
*/

// TODO: rename wheres to restr?

pub async fn get_projects_facet_count<'e, E>(
    ex: E,
    wheres: &[Facet]
) -> Result<i64, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut qb = QueryBuilder::new(
        "
SELECT COUNT(1)
FROM projects
        "
    );

    for wf in wheres {
        qb.push_join(wf);
    }

    qb.push(" WHERE ");

    let mut qbs = qb.separated(" AND ");
    for wf in wheres {
        qbs.push_where(wf);
    }

//    eprintln!("{}", qb.sql());

    Ok(
        qb
            .build_query_scalar()
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

/*
pub async fn get_projects_facet_query_count<'e, E>(
    ex: E,
    wheres: &[(&str, &str, &WhereValue<'_>)],
    query: &str,
) -> Result<i64, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let query = fts5_quote(query);

    let mut qb = QueryBuilder::new(
        "
SELECT COUNT(1)
FROM projects
JOIN projects_fts
ON projects_fts.rowid = projects.project_id
        "
    );

    // TODO: limit fields here

    add_joins_projects(&mut qb, wheres);

    qb.push(" WHERE projects_fts MATCH ")
        .push_bind(query)
        .push(" AND ");

    add_wheres(&mut qb, wheres);

    Ok(
        qb
            .build_query_scalar()
            .fetch_one(ex)
            .await?
    )
}
*/

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

const SUMMARY_FIELDS: &str = "
    projects.project_id,
    projects.name,
    projects.slug,
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
";

const WINDOW_SELECT: &str = formatcp!("
SELECT
    0.0 AS rank,
    {SUMMARY_FIELDS}
FROM projects
");

const WINDOW_SELECT_FTS: &str = formatcp!("
SELECT
    fts.rank,
    {SUMMARY_FIELDS}
FROM projects
");

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
        QueryBuilder::new(formatcp!("{WINDOW_SELECT} ORDER BY "))
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

pub async fn get_projects_facet_end_window<'e, E>(
    ex: E,
    wheres: &[Facet],
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut qb = QueryBuilder::new(WINDOW_SELECT);

    for wf in wheres {
        qb.push_join(wf);
    }

    qb.push(" WHERE ");

    let mut qbs = qb.separated(" AND ");
    for wf in wheres {
        qbs.push_where(wf);
    }

    Ok(
        qb
            .push(" ORDER BY ")
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
        QueryBuilder::new(formatcp!("{WINDOW_SELECT_FTS}
JOIN projects_fts AS fts
ON projects.project_id = fts.rowid
WHERE projects_fts MATCH
                "
            ))
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

/*
pub async fn get_projects_facet_query_end_window<'e, E>(
    ex: E,
    wheres: &[(&str, &str, &WhereValue<'_>)],
    query: &str,
    sort_by: SortBy,
    dir: Direction,
    limit: u32
) -> Result<Vec<ProjectSummaryRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut qb = QueryBuilder::new(formatcp!("{WINDOW_SELECT_FTS}
JOIN projects_fts AS fts
ON projects.project_id = fts.rowid
            "
        ));

    add_joins_fts(&mut qb, wheres);

    qb.push(" WHERE projects_fts MATCH ")
        .push_bind(fts5_quote(query))
        .push(" WHERE ");

    add_wheres(&mut qb, wheres);

    Ok(
        qb
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
*/

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
        QueryBuilder::new(formatcp!("{WINDOW_SELECT} WHERE "))
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

pub async fn get_projects_facet_mid_window<'e, 'f, E, F>(
    ex: E,
    wheres: &'f [Facet],
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
    let mut qb = QueryBuilder::new(WINDOW_SELECT);

    for wf in wheres {
        qb.push_join(wf);
    }

    qb.push(" WHERE ");

    let mut qbs = qb.separated(" AND ");
    for wf in wheres {
        qbs.push_where(wf);
    }

    Ok(
        qb
            .push(" AND (")
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
            .push(")) ORDER BY ")
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
        QueryBuilder::new(formatcp!("{WINDOW_SELECT_FTS}
JOIN (
    SELECT
        projects_fts.rowid,
        projects_fts.rank
    FROM projects_fts
    WHERE projects_fts MATCH
                "
            ))
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

/*
pub async fn get_projects_facet_query_mid_window<'e, 'f, E, F>(
    ex: E,
    wheres: &[(&str, &str, &'f WhereValue<'_>)],
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
    let mut qb = QueryBuilder::new(formatcp!("{WINDOW_SELECT_FTS}
JOIN (
    SELECT
        projects_fts.rowid,
        projects_fts.rank
    FROM projects_fts
    WHERE projects_fts MATCH
                "
            ));

    qb.push_bind(fts5_quote(query))
        .push(") AS fts ON fts.rowid = projects.project_id ");

    add_joins_projects(&mut qb, wheres);

    qb.push(" WHERE ");
    add_wheres(&mut qb, wheres);

    Ok(
        qb
            .push(" AND (")
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
            .push(")) ORDER BY ")
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
*/

#[cfg(test)]
mod test {
    use super::*;

    type Pool = sqlx::Pool<Sqlite>;

    #[test]
    fn fts5_quote_abc() {
        assert_eq!("\"abc\"", fts5_quote("abc"));
    }

    #[test]
    fn fts5_quote_abc_def() {
        assert_eq!("\"abc def\"", fts5_quote("abc def"));
    }

    #[test]
    fn fts5_quote_abcq_def() {
        assert_eq!("\"abc\"\" def\"", fts5_quote("abc\" def"));
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_count_ok(pool: Pool) {
        assert_eq!(get_projects_count(&pool).await.unwrap(), 2);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_query_count_one(pool: Pool) {
        assert_eq!(get_projects_query_count(&pool, "Another").await.unwrap(), 1);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_query_count_zero(pool: Pool) {
        assert_eq!(get_projects_query_count(&pool, "xxx").await.unwrap(), 0);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_count_one(pool: Pool) {
        let wheres = [
            Facet::Publisher("XYZ".into())
        ];
        assert_eq!(get_projects_facet_count(&pool, &wheres).await.unwrap(), 1);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_count_zero(pool: Pool) {
        let wheres = [
            Facet::Publisher("zzz".into())
        ];
        assert_eq!(get_projects_facet_count(&pool, &wheres).await.unwrap(), 0);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_count_multi_one(pool: Pool) {
        let wheres = [
            Facet::Publisher("XYZ".into()),
            Facet::Year("1993".into())
        ];
        assert_eq!(get_projects_facet_count(&pool, &wheres).await.unwrap(), 1);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_count_join_one(pool: Pool) {
        let wheres = [
            Facet::Publisher("XYZ".into()),
// TODO: bridge username to users over owners; argh
//            ("owners", "username", &WhereValue::Text("Alice".into()))
        ];
        assert_eq!(get_projects_facet_count(&pool, &wheres).await.unwrap(), 1);
    }

/*
    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_query_count_one(pool: Pool) {
        let wheres = [
            ("projects", "game_publisher", &WhereValue::Text("XYZ".into()))
        ];
        assert_eq!(get_projects_facet_query_count(&pool, &wheres, "Another").await.unwrap(), 0);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_query_count_query_zero(pool: Pool) {
        let wheres = [
            ("projects", "game_publisher", &WhereValue::Text("XYZ".into()))
        ];
        assert_eq!(get_projects_facet_query_count(&pool, &wheres, "xxx").await.unwrap(), 0);
    }

    #[sqlx::test(fixtures("users", "projects"))]
    async fn get_projects_facet_query_count_facet_zero(pool: Pool) {
        let wheres = [
            ("projects", "game_publisher", &WhereValue::Text("zzz".into()))
        ];
        assert_eq!(get_projects_facet_query_count(&pool, &wheres, "Another").await.unwrap(), 0);
    }
*/

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
