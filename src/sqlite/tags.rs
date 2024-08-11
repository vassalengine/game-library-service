use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    core::CoreError,
    model::Project
};

pub async fn get_tags<'e, E>(
    ex: E,
    proj: Project
) -> Result<Vec<String>, CoreError>
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

pub async fn get_tags_at<'e, E>(
    ex: E,
    proj: Project,
   date: i64
) -> Result<Vec<String>, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(vec![])
}
