use sqlx::{
    Executor, Transaction,
    sqlite::Sqlite
};

use crate::{
    errors::AppError,
    model::{User, Users}
};

pub struct Database {
    database: SqlitePool
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

pub async fn user_is_owner<'e, E>(
    user: &str,
    proj_id: u32,
    ex: E
) -> Result<bool, AppError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 as present
FROM owners
JOIN users
ON users.id = owners.user_id
WHERE users.username = ? AND owners.project_id = ?
LIMIT 1
            ",
            user,
            proj_id
        )
        .fetch_optional(ex)
        .await?
        .is_some()
    )
}

pub async fn add_owner(
    user_id: i64,
    proj_id: u32,
    tx: &mut Transaction<'_, Sqlite>
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
INSERT OR IGNORE INTO owners (user_id, project_id)
VALUES (?, ?)
        ",
        user_id,
        proj_id
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn remove_owner(
    user_id: i64,
    proj_id: u32,
    tx: &mut Transaction<'_, Sqlite>
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
DELETE FROM owners
WHERE user_id = ? AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn get_owners<'e, E>(
    proj_id: u32,
    ex: E
) -> Result<Users, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>
{
    let users = sqlx::query_as!(
            User,
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
    .fetch_all(ex)
    .await?;

    Ok(Users {
        users: users 
    })
}

pub async fn get_user_id<'e, E>(
    user: &str,
    ex: E
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT id
FROM users
WHERE username = ?
            ",
            user 
        )
        .fetch_one(ex)
        .await?
        .id
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test]
    async fn test_x() {
     
    }
}
