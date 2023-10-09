use sqlx::{
    Executor,
    sqlite::{Sqlite, SqlitePool}
};

use crate::{
    errors::AppError,
    model::{User, Users}
};

#[derive(Clone)]
pub struct Database(pub SqlitePool);

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

async fn get_user_id(
    user: &str,
    db: &Database
) -> Result<i64, sqlx::Error> {
    Ok(
        sqlx::query!(
            "
SELECT id
FROM users
WHERE username = ?
            ",
            user
        )
        .fetch_one(&db.0)
        .await?
        .id
    )
}

pub async fn user_is_owner(
    user: &str,
    proj_id: u32,
    db: &Database
) -> Result<bool, AppError>
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
        .fetch_optional(&db.0)
        .await?
        .is_some()
    )
}

async fn add_owner<'e, E>(
    user_id: i64,
    proj_id: u32,
    ex: E
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT OR IGNORE INTO owners (user_id, project_id)
VALUES (?, ?)
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn add_owners(
    owners: &[String],
    proj_id: u32,
    db: &Database
) -> Result<(), AppError>
{
    let mut tx = db.0.begin().await?;

    for owner in owners {
        // get user id of new owner
        let owner_id = get_user_id(owner, db).await?;
        // associate new owner with the project
        add_owner(owner_id, proj_id, &mut *tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn remove_owner<'e, E>(
    user_id: i64,
    proj_id: u32,
    ex: E
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
DELETE FROM owners
WHERE user_id = ? AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn remove_owners(
    owners: &[String],
    proj_id: u32,
    db: &Database
) -> Result<(), AppError>
{
    let mut tx = db.0.begin().await?;

    for owner in owners {
        // get user id of owner
        let owner_id = get_user_id(owner, db).await?;
        // remove old owner from the project
        remove_owner(owner_id, proj_id, &mut *tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn get_owners(
    proj_id: u32,
    db: &Database
) -> Result<Users, AppError>
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
    .fetch_all(&db.0)
    .await?;

    Ok(Users {
        users
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(fixtures("user"))]
    async fn test_get_user_id_present(pool: SqlitePool) {
        let db = Database(pool);
        assert_eq!(get_user_id("bob", &db).await.unwrap(), 1);
    }

    #[sqlx::test(fixtures("user"))]
    async fn test_get_user_id_missing(pool: SqlitePool) {
        let db = Database(pool);
        assert!(get_user_id("not_a_user", &db).await.is_err());
    }
}
