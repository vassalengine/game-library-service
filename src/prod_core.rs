use axum::async_trait;
use chrono::Utc;
use semver::Version;
use sqlx::Executor;

use crate::{
    core::Core,
    errors::AppError,
    model::{GameData, Project, ProjectData, ProjectDataPut, ProjectID, Readme, User, Users}
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

// TODO: switch proj_id to proj_name; then we will always know if the project
// exists because we have to look up the id

#[async_trait]
impl Core for ProdCore {
    async fn get_project_id(
         &self,
        proj: &Project
    ) -> Result<ProjectID, AppError>
    {
        sqlx::query_scalar!(
            "
SELECT id
FROM projects
WHERE name = ?
            ",
            proj.0
        )
        .fetch_optional(&self.db)
        .await?
        .map(ProjectID)
        .ok_or(AppError::NotAProject)
    }

    async fn get_owners(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        Ok(
            Users {
                users: sqlx::query_scalar!(
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
                .collect()
            }
        )
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: i64
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
        proj_id: i64
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
            return Err(AppError::CannotRemoveLastOwner);
        }

        tx.commit().await?;

        Ok(())
    }

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: i64
    ) -> Result<bool, AppError>
    {
        Ok(
            sqlx::query!(
                "
SELECT 1 AS present
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

    async fn get_project(
        &self,
        proj_id: i64,
    ) -> Result<ProjectData, AppError>
    {
        let row = sqlx::query!(
            "
SELECT
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
FROM projects
WHERE id == ?
LIMIT 1
            ",
            proj_id
         )
        .fetch_one(&self.db)
        .await?;

        Ok(
            ProjectData {
                name: row.name,
                description: row.description,
                revision: row.revision,
                created_at: row.created_at,
                modified_at: row.modified_at,
                tags: Vec::new(),
                game: GameData {
                    title: row.game_title,
                    title_sort_key: row.game_title_sort,
                    publisher: row.game_publisher,
                    year: row.game_year
                },
                owners: Vec::new(),
                packages: Vec::new()
            }
        )
    }

// TODO: require project names to match [A-Za-z][A-Za-z0-9_-]{,63}?
// TODO: maybe also compare case-insensitively and equate - and _?
// TODO: length limits on strings

    async fn create_project(
        &self,
        user: &User,
        proj: &str,
        proj_data: &ProjectDataPut
    ) -> Result<(), AppError>
    {
        let now = Utc::now().to_rfc3339();
        let game_title_sort_key = title_sort_key(&proj_data.game.title);

        let mut tx = self.db.begin().await?;

        let proj_id = sqlx::query_scalar!(
            "
INSERT INTO projects (
    name,
    description,
    revision,
    created_at,
    modified_at,
    game_title,
    game_title_sort,
    game_publisher,
    game_year
)
VALUES (?, ?, 1, ?, ?, ?, ?, ?, ?)
RETURNING id
            ",
            proj,
            proj_data.description,
            now,
            now,
            proj_data.game.title,
            game_title_sort_key,
            proj_data.game.publisher,
            proj_data.game.year
        )
        .fetch_one(&mut *tx)
        .await?;

        // get user id of new owner
        let owner_id = get_user_id(&user.0, &self.db).await?;

        // associate new owner with the project
        add_owner(owner_id, proj_id, &mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_project(
        &self,
        proj_id: i64,
        proj_data: &ProjectDataPut
    ) -> Result<(), AppError>
    {
        let now = Utc::now().to_rfc3339();

        let mut tx = self.db.begin().await?;

        // archive the previous revision
        let revision = 1 + sqlx::query_scalar!(
            "
INSERT INTO projects_revisions
SELECT *
FROM projects
WHERE projects.id = ?
RETURNING revision
            ",
            proj_id
         )
        .fetch_one(&mut *tx)
        .await?;

        // update to the current revision
        sqlx::query!(
            "
UPDATE projects
SET
    description = ?,
    revision = ?,
    modified_at = ?,
    game_title = ?,
    game_title_sort = ?,
    game_publisher = ?,
    game_year = ?
WHERE id = ?
            ",
            proj_data.description,
            revision,
            now,
            proj_data.game.title,
            proj_data.game.title_sort_key,
            proj_data.game.publisher,
            proj_data.game.year,
            proj_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_package(
        &self,
        _proj_id: i64,
        pkg_id: i64
    ) -> Result<String, AppError>
    {
        sqlx::query_scalar!(
            "
SELECT url
FROM package_versions
WHERE package_id = ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC
LIMIT 1
            ",
            pkg_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotAVersion)
    }

    async fn get_package_version(
        &self,
        _proj_id: i64,
        pkg_id: i64,
        version: &str
    ) -> Result<String, AppError>
    {
        let version = parse_version(version)?;

        sqlx::query_scalar!(
            "
SELECT url
FROM package_versions
WHERE package_id = ?
    AND version_major = ?
    AND version_minor = ?
    AND version_patch = ?
LIMIT 1
            ",
            pkg_id,
            version.0,
            version.1,
            version.2
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotAVersion)
    }

    async fn get_players(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        Ok(
            Users {
                users: sqlx::query_scalar!(
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
                .collect()
            }
        )
    }

    async fn add_player(
        &self,
        player: &User,
        proj_id: i64
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
        proj_id: i64
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

    async fn get_readme(
        &self,
        proj_id: i64
    ) -> Result<Readme, AppError>
    {
        Ok(
            sqlx::query_as!(
                Readme,
                "
SELECT text
FROM readmes
WHERE project_id = ?
ORDER BY revision DESC
LIMIT 1
                ",
                proj_id
            )
            .fetch_one(&self.db)
            .await?
        )
    }

    async fn get_readme_revision(
        &self,
        proj_id: i64,
        revision: u32
    ) -> Result<Readme, AppError>
    {
        sqlx::query_as!(
            Readme,
            "
SELECT text
FROM readmes
WHERE project_id = ?
AND revision = ?
LIMIT 1
            ",
            proj_id,
            revision
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotARevision)
    }
}

// TODO: check pre and build fields of Version

fn try_vtup(v: Version) -> Result<(i64, i64, i64), AppError>  {
    Ok(
        (
            i64::try_from(v.major).or(Err(AppError::MalformedVersion))?,
            i64::try_from(v.minor).or(Err(AppError::MalformedVersion))?,
            i64::try_from(v.patch).or(Err(AppError::MalformedVersion))?
        )
    )
}

fn parse_version(version: &str) -> Result<(i64, i64, i64), AppError> {
    Version::parse(version)
        .or(Err(AppError::MalformedVersion))
        .and_then(try_vtup)
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
    proj_id: i64,
    ex: E
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Database>
{
    sqlx::query!(
        "
INSERT OR IGNORE INTO owners (
    user_id,
    project_id
)
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
    proj_id: i64,
    ex: E
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Database>
{
    sqlx::query!(
        "
DELETE FROM owners
WHERE user_id = ?
    AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn has_owner<'e, E>(
    proj_id: i64,
    ex: E
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Database>
{
    Ok(
        sqlx::query!(
            "
SELECT 1 AS present
FROM owners
WHERE project_id = ?
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
    proj_id: i64,
    ex: E
) -> Result<(), AppError>
where
    E: Executor<'e, Database = Database>
{
    sqlx::query!(
        "
INSERT OR IGNORE INTO players (
    user_id,
    project_id
)
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
    proj_id: i64,
    ex: E
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Database>
{
    sqlx::query!(
        "
DELETE FROM players
WHERE user_id = ?
    AND project_id = ?
        ",
        user_id,
        proj_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

fn split_title_sort_key<'a>(title: &'a str) -> (&'a str, Option<&'a str>) {
    match title.split_once(' ') {
        // Probably Spanish or French, "A" is not an article
        Some(("A", rest)) if rest.starts_with("la") => (title, None),
        // Put leading article at end
        Some(("A", rest)) => (rest, Some("A")),
        Some(("An", rest)) => (rest, Some("An")),
        Some(("The", rest)) => (rest, Some("The")),
        // Doesn't start with an article
        Some(_) | None => (title, None)
    }
}

fn title_sort_key(title: &str) -> String {
    match split_title_sort_key(title) {
        (_, None) => title.into(),
        (rest, Some(art)) => format!("{rest}, {art}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(fixtures("projects", "users"))]
    async fn create_project_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        let user = User("bob".into());
        let data = ProjectDataPut {
            description: "".into(),
            tags: vec!(),
            game: GameData {
                title: "".into(),
                title_sort_key: "".into(),
                publisher: "".into(),
                year: "".into()
            }
        };

        core.create_project(&user, "newproj", &data).await.unwrap();
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_package(42, 1).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_version_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_package_version(42, 1, "1.2.3").await.unwrap(),
            "https://example.com/a_package-1.2.3"
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_version_malformed(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_package_version(42, 1, "xyzzy").await.unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_version_not_a_version(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_package_version(42, 1, "1.0.0").await.unwrap_err(),
            AppError::NotAVersion
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec!(User("bob".into())) }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        let core = ProdCore { db: pool };
        assert!(core.user_is_owner(&User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        let core = ProdCore { db: pool };
        assert!(!core.user_is_owner(&User("alice".into()), 42).await.unwrap());
    }

// TODO: add test for non-user owner

    #[sqlx::test(fixtures("projects", "one_owner"))]
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

    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn remove_owners_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("bob".into())) };
        core.remove_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec!(User("alice".into())) }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn remove_owners_fail_if_last(pool: Pool) {
        let core = ProdCore { db: pool };
        let users = Users { users: vec!(User("bob".into())) };
        assert_eq!(
            core.remove_owners(&users, 1).await.unwrap_err(),
            AppError::CannotRemoveLastOwner
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
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

    #[sqlx::test(fixtures("projects", "players"))]
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

    #[sqlx::test(fixtures("projects", "players"))]
    async fn remove_player_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        core.remove_player(&User("bob".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users { users: vec!(User("alice".into())) }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_readme(42).await.unwrap(),
            Readme { text: "third try".into() }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_revision_ok(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_readme_revision(42, 2).await.unwrap(),
            Readme { text: "second try".into() }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_revision_bad(pool: Pool) {
        let core = ProdCore { db: pool };
        assert_eq!(
            core.get_readme_revision(42, 4).await.unwrap_err(),
            AppError::NotARevision
        );
    }
}
