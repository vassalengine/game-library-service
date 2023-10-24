use axum::async_trait;
use sqlx::Executor;

use crate::{
    core::Core,
    errors::AppError,
    model::{User, Users}
};

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

type Database = sqlx::sqlite::Sqlite;
type Pool = sqlx::Pool<Database>;

#[derive(Clone)]
pub struct ProdCore {
    pub db: Pool
}

#[async_trait]
impl Core for ProdCore {
    async fn get_owners(
        &self,
        proj_id: u32
    ) -> Result<Users, AppError>
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
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(User)
        .collect();

        Ok(Users { users })
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        let mut tx = self.db.begin().await?;

        for owner in &owners.users {
            // get user id of new owner
            let owner_id = get_user_id(&owner.0, &self.db).await?;
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
    ) -> Result<(), AppError>
    {
        let mut tx = self.db.begin().await?;

        for owner in &owners.users {
            // get user id of owner
            let owner_id = get_user_id(&owner.0, &self.db).await?;
            // remove old owner from the project
            remove_owner(owner_id, proj_id, &mut *tx).await?;
        }

        // prevent removal of last owner
        if !has_owner(proj_id, &mut *tx).await? {
            return Err(AppError::DatabaseError("cannot remove last owner".into()));
        }

        tx.commit().await?;

        Ok(())
    }

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: u32
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
                user.0,
                proj_id
            )
            .fetch_optional(&self.db)
            .await?
            .is_some()
        )
    }

    async fn get_players(
        &self,
        proj_id: u32
    ) -> Result<Users, AppError>
    {
// FIXME: Is this really the best way? Can't we fill users directly?
        let users = sqlx::query_scalar!(
            "
SELECT users.username
FROM users
JOIN players
ON users.id = players.user_id
JOIN projects
ON players.project_id = projects.id
WHERE projects.id = ?
ORDER BY users.username
            ",
            proj_id
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(User)
        .collect();

        Ok(Users { users })
    }

    async fn add_player(
        &self,
        player: &User,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        let mut tx = self.db.begin().await?;

        // get user id of new player
        let player_id = get_user_id(&player.0, &self.db).await?;
        // associate new player with the project
        add_player(player_id, proj_id, &mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn remove_player(
        &self,
        player: &User,
        proj_id: u32
    ) -> Result<(), AppError>
    {
        let mut tx = self.db.begin().await?;

        // get user id of player
        let player_id = get_user_id(&player.0, &self.db).await?;
        // remove player from the project
        remove_player(player_id, proj_id, &mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

async fn get_user_id(
    user: &str,
    db: &Pool
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
        .fetch_one(db)
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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
    E: Executor<'e, Database = Database>
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

async fn add_player<'e, E>(
    user_id: i64,
    proj_id: u32,
    ex: E
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Database>
{
    sqlx::query!(
        "
INSERT OR IGNORE INTO players (user_id, project_id)
VALUES (?, ?)
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn remove_player<'e, E>(
    user_id: i64,
    proj_id: u32,
    ex: E
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Database>
{
    sqlx::query!(
        "
DELETE FROM players
WHERE user_id = ? AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(fixtures("one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec!(User("bob".into())) }
        );
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn get_owners_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        // FIXME: should this be an error?
        assert_eq!(
            core.get_owners(1).await.unwrap(),
            Users { users: Vec::new() }
        );
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        let core = ProdCore { db: pool };
        assert!(core.user_is_owner(&User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        let core = ProdCore { db: pool };
        assert!(!core.user_is_owner(&User("alice".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn user_is_owner_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        // FIXME: should this be an error?
        assert!(!core.user_is_owner(&User("bob".into()), 1).await.unwrap());
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn add_owners_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("alice".into())) };
        core.add_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users {
                users: vec!(
                    User("alice".into()),
                    User("bob".into())
                )
            }
        );
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn add_owners_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("bob".into())) };
        assert!(core.add_owners(&users, 1).await.is_err());
    }

    #[sqlx::test(fixtures("two_owners"))]
    async fn remove_owners_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("bob".into())) };
        core.remove_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec!(User("alice".into())) }
        );
    }

    #[sqlx::test(fixtures("one_owner"))]
    async fn remove_owners_fail_if_last(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("bob".into())) };
        assert!(core.remove_owners(&users, 1).await.is_err());
    }

    #[sqlx::test(fixtures("two_owners"))]
    async fn remove_owners_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("bob".into())) };
        assert!(core.remove_owners(&users, 1).await.is_err());
    }

    #[sqlx::test(fixtures("players"))]
    async fn get_players_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users {
                users: vec!(
                    User("alice".into()),
                    User("bob".into())
                )
            }
        );
    }

    #[sqlx::test(fixtures("players"))]
    async fn get_players_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        // FIXME: should this be an error?
        assert_eq!(
            core.get_players(1).await.unwrap(),
            Users { users: Vec::new() }
        );
    }

    #[sqlx::test(fixtures("players"))]
    async fn add_player_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        core.add_player(&User("chuck".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users {
                users: vec!(
                    User("alice".into()),
                    User("bob".into()),
                    User("chuck".into())
                )
            }
        );
    }

    #[sqlx::test(fixtures("players"))]
    async fn add_player_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        assert!(core.add_player(&User("chuck".into()), 1).await.is_err());
    }

    #[sqlx::test(fixtures("players"))]
    async fn remove_player_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        core.remove_player(&User("bob".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users { users: vec!(User("alice".into())) }
        );
    }

    #[sqlx::test(fixtures("players"))]
    async fn remove_player_not_a_project(pool: Pool) {
        let core = ProdCore { db: pool };
        // FIXME: should this be an error?
        core.remove_player(&User("bob".into()), 1).await.unwrap();
    }
}
