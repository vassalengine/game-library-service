use sqlx::{
    Executor,
    sqlite::Sqlite
};

use crate::{
    db::DatabaseError
};

pub async fn get_publishers<'e, E>(
    ex: E
) -> Result<Vec<String>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT DISTINCT game_publisher
FROM projects
ORDER BY game_publisher COLLATE NOCASE
            "
        )
        .fetch_all(ex)
        .await?
    )
}
