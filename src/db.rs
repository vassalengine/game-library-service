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
