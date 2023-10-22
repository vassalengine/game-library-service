use axum::async_trait;
use sqlx::{
    Executor,
    sqlite::{Sqlite, SqlitePool}
};

use crate::{
    datastore::{DataStore, DataStoreError},
    model::{User, Users}
};

#[derive(Clone)]
pub struct Database(pub SqlitePool);

#[async_trait]
impl DataStore for Database {
    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: u32
    ) -> Result<bool, DataStoreError>
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
                user.0,
                proj_id
            )
            .fetch_optional(&self.0)
            .await?
            .is_some()
        )
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), DataStoreError>
    {
        let mut tx = self.0.begin().await?;

        for owner in &owners.users {
            // get user id of new owner
            let owner_id = get_user_id(&owner.0, self).await?;
            // associate new owner with the project
            add_owner(owner_id, proj_id, &mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), DataStoreError>
    {
        let mut tx = self.0.begin().await?;

        for owner in &owners.users {
            // get user id of owner
            let owner_id = get_user_id(&owner.0, self).await?;
            // remove old owner from the project
            remove_owner(owner_id, proj_id, &mut *tx).await?;
        }

        // prevent removal of last owner 
        if !has_owner(proj_id, &mut *tx).await? {
            return Err(DataStoreError::Problem("cannot remove last owner".into()));
        }

        tx.commit().await?;

        Ok(())
    }

    async fn get_owners(
        &self,
        proj_id: u32
    ) -> Result<Users, DataStoreError>
    {
// FIXME: Is this really the best way? Can't we fill users directly?
        let users = sqlx::query_scalar!(
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
        .fetch_all(&self.0)
        .await?
        .into_iter()
        .map(User)
        .collect();

        Ok(Users { users })
    }
}

impl From<sqlx::Error> for DataStoreError {
    fn from(e: sqlx::Error) -> Self {
        DataStoreError::Problem(e.to_string())
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

async fn has_owner<'e, E>(
    proj_id: u32,
    ex: E
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 as present
FROM owners
WHERE owners.project_id = ?
LIMIT 1
            ",
            proj_id
        )
        .fetch_optional(ex)
        .await?
        .is_some()
    )
}

#[cfg(test)]
mod test {
    use super::*;

    async fn user_id_is_owner<'e, E>(
        user_id: i64,
        proj_id: u32,
        ex: E
    ) -> Result<bool, sqlx::Error>
    where
        E: Executor<'e, Database = Sqlite>
    {
        Ok(
            sqlx::query!(
                "
SELECT 1 as present
FROM owners
WHERE user_id = ? AND project_id = ?
LIMIT 1
                ",
                user_id,
                proj_id
            )
            .fetch_optional(ex)
            .await?
            .is_some()
        )
    }

    async fn user_id_exists<'e, E>(
        user_id: i64,
        ex: E
    ) -> Result<bool, sqlx::Error>
    where
        E: Executor<'e, Database = Sqlite>
    {
        Ok(
            sqlx::query!(
                "
SELECT 1 as present
FROM users
WHERE id = ?
LIMIT 1
                ",
                user_id,
            )
            .fetch_optional(ex)
            .await?
            .is_some()
        )
    }

    async fn project_id_exists<'e, E>(
        proj_id: i64,
        ex: E
    ) -> Result<bool, sqlx::Error>
    where
        E: Executor<'e, Database = Sqlite>
    {
        Ok(
            sqlx::query!(
                "
SELECT 1 as present
FROM projects 
WHERE id = ?
LIMIT 1
                ",
                proj_id,
            )
            .fetch_optional(ex)
            .await?
            .is_some()
        )
    }

    #[sqlx::test(fixtures("user"))]
    async fn get_user_id_present(pool: SqlitePool) {
        let db = Database(pool);
        assert_eq!(get_user_id("bob", &db).await.unwrap(), 1);
    }

    #[sqlx::test(fixtures("user"))]
    async fn get_user_id_missing(pool: SqlitePool) {
        let db = Database(pool);
        assert!(get_user_id("not_a_user", &db).await.is_err());
    }

    #[sqlx::test(fixtures("owner"))]
    async fn add_owner_new(pool: SqlitePool) {
        assert!(!user_id_is_owner(2, 42, &pool).await.unwrap()); 
        assert!(add_owner(2, 42, &pool).await.is_ok());
        assert!(user_id_is_owner(2, 42, &pool).await.unwrap()); 
    }

    #[sqlx::test(fixtures("owner"))]
    async fn add_owner_existing(pool: SqlitePool) {
        assert!(user_id_is_owner(1, 42, &pool).await.unwrap()); 
        assert!(add_owner(1, 42, &pool).await.is_ok());
        assert!(user_id_is_owner(1, 42, &pool).await.unwrap()); 
    }

    #[sqlx::test(fixtures("owner"))]
    async fn add_owner_not_a_user(pool: SqlitePool) {
        assert!(!user_id_exists(3, &pool).await.unwrap());
        assert!(add_owner(3, 42, &pool).await.is_err());
    }

    #[sqlx::test(fixtures("owner"))]
    async fn add_owner_not_a_project(pool: SqlitePool) {
        assert!(!project_id_exists(1, &pool).await.unwrap());
        assert!(add_owner(1, 1, &pool).await.is_err());
    }

    #[sqlx::test(fixtures("owner"))]
    async fn remove_owner_existing(pool: SqlitePool) {
        assert!(user_id_is_owner(1, 42, &pool).await.unwrap()); 
        assert!(remove_owner(1, 42, &pool).await.is_ok());
        assert!(!user_id_is_owner(1, 42, &pool).await.unwrap()); 
    }

    #[sqlx::test(fixtures("owner"))]
    async fn remove_owner_non_owner(pool: SqlitePool) {
        assert!(!user_id_is_owner(2, 42, &pool).await.unwrap()); 
        assert!(remove_owner(2, 42, &pool).await.is_ok());
        assert!(!user_id_is_owner(2, 42, &pool).await.unwrap()); 
    }

    #[sqlx::test(fixtures("owner"))]
    async fn remove_owner_not_a_user(pool: SqlitePool) {
        // removal of nonexistent user is ok
        assert!(remove_owner(3, 42, &pool).await.is_ok());
    }

    #[sqlx::test(fixtures("owner"))]
    async fn remove_owner_not_a_project(pool: SqlitePool) {
        // removal from nonexistent project is ok
        assert!(remove_owner(1, 1, &pool).await.is_ok());
    }

/*
    #[sqlx::test(fixtures("owner"))]
    async fn remove_owner_last(pool: SqlitePool) {
// HERE        
        assert!(user_id_is_owner(1, 42, &pool).await.unwrap()); 
        assert!(remove_owner(1, 42, &pool).await.is_err());
    }
*/

    #[sqlx::test(fixtures("owner"))]
    async fn get_owners_ok(pool: SqlitePool) {
        let db = Database(pool);
        assert_eq!(
            db.get_owners(42).await.unwrap(),
            Users { users: vec!(User("bob".into())) }
        );
    }

    #[sqlx::test(fixtures("owner"))]
    async fn get_owners_not_a_project(pool: SqlitePool) {
        let db = Database(pool);
        assert_eq!(
            db.get_owners(1).await.unwrap(),
            Users { users: Vec::new() }
        );
    }

    // TODO: prevent removal of last owner
}
