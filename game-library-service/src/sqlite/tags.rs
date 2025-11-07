use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    db::DatabaseError,
    model::Project
};

pub async fn get_project_tags<'e, E>(
    ex: E,
    proj: Project
) -> Result<Vec<String>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT
    tag
FROM tags
WHERE project_id = ?
            ",
            proj.0
        )
        .fetch_all(ex)
        .await?
    )
}

pub async fn get_project_tags_at<'e, E>(
    ex: E,
    proj: Project,
   date: i64
) -> Result<Vec<String>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
// TODO
    Ok(vec![])
}

pub async fn get_tags<'e, E>(
    ex: E,
) -> Result<Vec<String>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT DISTINCT tag
FROM tags
ORDER BY tag COLLATE NOCASE
            "
        )
        .fetch_all(ex)
        .await?
    )
}
