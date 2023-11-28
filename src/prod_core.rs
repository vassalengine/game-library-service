use axum::async_trait;
use chrono::{DateTime, Utc};
use semver::Version;

use crate::{
    core::Core,
    errors::AppError,
    model::{GameData, PackageData, Project, ProjectData, ProjectDataPut, ProjectID, Projects, ProjectSummary, Readme, User, Users, VersionData},
    pagination::{Limit, Pagination, Seek, SeekLink},
    sql::{self, Pool}
};

#[derive(Clone)]
pub struct ProdCore {
    pub db: Pool,
    pub now: fn() -> DateTime<Utc>
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
        sql::get_project_id(&self.db, &proj.0).await
    }

    async fn get_owners(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        sql::get_owners(&self.db, proj_id).await
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
            let owner_id = sql::get_user_id(&self.db, &owner.0).await?;
            // associate new owner with the project
            sql::add_owner(&mut *tx, owner_id, proj_id).await?;
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
            let owner_id = sql::get_user_id(&self.db, &owner.0).await?;
            // remove old owner from the project
            sql::remove_owner(&mut *tx, owner_id, proj_id).await?;
        }

        // prevent removal of last owner
        if !sql::has_owner(&mut *tx, proj_id).await? {
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
        sql::user_is_owner(&self.db, user, proj_id).await
    }

    async fn get_projects(
        &self,
        from: Seek,
        limit: Limit
    ) -> Result<Projects, AppError>
    {
        let limit = limit.get() as u32;

        let (prev_page, next_page, projects) = match from {
            Seek::Start => get_projects_start(limit, &self.db).await?,
            Seek::After(name) => get_projects_after(&name, limit, &self.db).await?,
            Seek::Before(name) => get_projects_before(&name, limit, &self.db).await?,
            Seek::End => get_projects_end(limit, &self.db).await?
        };

        Ok(
            Projects {
                projects,
                meta: Pagination {
                    prev_page,
                    next_page,
                    total: sql::get_project_count(&self.db).await?
                }
            },
        )
    }

    async fn get_project(
        &self,
        proj_id: i64,
    ) -> Result<ProjectData, AppError>
    {
        let proj_row = sql::get_project_row(&self.db, proj_id).await?;

        let owners = self.get_owners(proj_id)
            .await?
            .users
            .into_iter()
            .map(|u| u.0)
            .collect();

// TODO: gross, is there a better way to do this?
        let package_rows = sql::get_packages(&self.db, proj_id).await?;

        let mut packages = Vec::with_capacity(package_rows.len());

        for pr in package_rows {
            let versions = sql::get_versions(&self.db, pr.id)
                .await?
                .into_iter()
                .map(|vr| VersionData {
                    version: vr.version,
                    filename: vr.filename,
                    url: vr.url,
                    size: 0,
                    checksum: "".into(),
                    published_at: "".into(),
                    published_by: "".into(),
                    requires: "".into(),
// TODO: get authors
                    authors: vec![]
                })
                .collect();

            packages.push(
                PackageData {
                    name: pr.name,
                    description: "".into(),
                    versions
                }
            );
        }

        Ok(
            ProjectData {
                name: proj_row.name,
                description: proj_row.description,
                revision: proj_row.revision,
                created_at: proj_row.created_at,
                modified_at: proj_row.modified_at,
                tags: vec![],
                game: GameData {
                    title: proj_row.game_title,
                    title_sort_key: proj_row.game_title_sort,
                    publisher: proj_row.game_publisher,
                    year: proj_row.game_year
                },
                owners,
                packages
            }
        )
    }

// TODO: require project names to match [A-Za-z0-9][A-Za-z0-9_-]{,63}?
// TODO: maybe also compare case-insensitively and equate - and _?
// TODO: length limits on strings
// TODO: require package names to match [A-Za-z0-9][A-Za-z0-9_-]{,63}?
// TODO: packages might need display names?

    async fn create_project(
        &self,
        user: &User,
        proj: &str,
        proj_data: &ProjectDataPut
    ) -> Result<(), AppError>
    {
        let now = (self.now)().to_rfc3339();
        let game_title_sort_key = title_sort_key(&proj_data.game.title);

        let mut tx = self.db.begin().await?;

        let proj_id = sql::create_project(
            &mut *tx,
            proj,
            proj_data,
            &game_title_sort_key,
            &now
        ).await?;

        // get user id of new owner
        let owner_id = sql::get_user_id(&self.db, &user.0).await?;

        // associate new owner with the project
        sql::add_owner(&mut *tx, owner_id, proj_id).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_project(
        &self,
        proj_id: i64,
        proj_data: &ProjectDataPut
    ) -> Result<(), AppError>
    {
        let now = (self.now)().to_rfc3339();

        let mut tx = self.db.begin().await?;

        // archive the previous revision
        let revision = 1 + sql::copy_project_revision(&mut *tx, proj_id).await?;

        // update to the current revision
        sql::update_project(&mut *tx, proj_id, revision, proj_data, &now).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_project_revision(
        &self,
        proj_id: i64,
        revision: u32
    ) -> Result<ProjectData, AppError>
    {
        let proj_row = sql::get_project_row_revision(
            &self.db, proj_id, revision
        ).await?;

        let owners = self.get_owners(proj_id)
            .await?
            .users
            .into_iter()
            .map(|u| u.0)
            .collect();

        Ok(
            ProjectData {
                name: proj_row.name,
                description: proj_row.description,
                revision: proj_row.revision,
                created_at: proj_row.created_at,
                modified_at: proj_row.modified_at,
                tags: Vec::new(),
                game: GameData {
                    title: proj_row.game_title,
                    title_sort_key: proj_row.game_title_sort,
                    publisher: proj_row.game_publisher,
                    year: proj_row.game_year
                },
                owners,
// TODO
                packages: vec![]
            }
        )
    }

    async fn get_package(
        &self,
        _proj_id: i64,
        pkg_id: i64
    ) -> Result<String, AppError>
    {
        sql::get_package_url(&self.db, pkg_id).await
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
        sql::get_players(&self.db, proj_id).await
    }

    async fn add_player(
        &self,
        player: &User,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        let mut tx = self.db.begin().await?;

        // get user id of new player
        let player_id = sql::get_user_id(&self.db, &player.0).await?;
        // associate new player with the project
        sql::add_player(&mut *tx, player_id, proj_id).await?;

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
        let player_id = sql::get_user_id(&self.db, &player.0).await?;
        // remove player from the project
        sql::remove_player(&mut *tx, player_id, proj_id).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_readme(
        &self,
        proj_id: i64
    ) -> Result<Readme, AppError>
    {
        sql::get_readme(&self.db, proj_id).await
    }

    async fn get_readme_revision(
        &self,
        proj_id: i64,
        revision: u32
    ) -> Result<Readme, AppError>
    {
        sql::get_readme_revision(&self.db, proj_id, revision).await
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

/*
async fn get_authors(
    db: &Pool,
    pkg_id: i64
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
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
*/

async fn get_projects_start(
    limit: u32,
    db: &Pool
) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
{
    // try to get one extra so we can tell if we're at an endpoint
    let limit_extra = limit + 1;

    let mut projects = sql::get_projects_start_window(db, limit_extra).await?;

    Ok(
        match projects.len() {
            l if l == limit_extra as usize => {
                projects.pop();
                (
                    None,
                    Some(SeekLink::new(Seek::After(projects[projects.len() - 1].name.clone()))),
                    projects
                )
            }
            _ => {
                (
                    None,
                    None,
                    projects
                )
            }
        }
    )
}

async fn get_projects_end(
    limit: u32,
    db: &Pool
) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
{
    // try to get one extra so we can tell if we're at an endpoint
    let limit_extra = limit + 1;

    let mut projects = sql::get_projects_end_window(db, limit_extra).await?;

    Ok(
        if projects.len() == limit_extra as usize {
            projects.pop();
            projects.reverse();
            (
                Some(SeekLink::new(Seek::Before(projects[0].name.clone()))),
                None,
                projects
            )
        }
        else {
            projects.reverse();
            (
                None,
                None,
                projects
            )
        }
    )
}

async fn get_projects_after(
    name: &str,
    limit: u32,
    db: &Pool
) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
{
    // try to get one extra so we can tell if we're at an endpoint
    let limit_extra = limit + 1;

    let mut projects = sql::get_projects_after_window(db, name, limit_extra)
        .await?;

    Ok(
        if projects.len() == limit_extra as usize {
            projects.pop();
            (
                Some(SeekLink::new(Seek::Before(projects[0].name.clone()))),
                Some(SeekLink::new(Seek::After(projects[projects.len() - 1].name.clone()))),
                projects
            )
        }
        else if projects.is_empty() {
            (
                Some(SeekLink::new(Seek::End)),
                None,
                projects
            )
        }
        else {
            (
                Some(SeekLink::new(Seek::Before(projects[0].name.clone()))),
                None,
                projects
            )
        }
    )
}

async fn get_projects_before(
    name: &str,
    limit: u32,
    db: &Pool
) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
{
    // try to get one extra so we can tell if we're at an endpoint
    let limit_extra = limit + 1;

    let mut projects = sql::get_projects_before_window(db, name, limit_extra)
        .await?;

    Ok(
        if projects.len() == limit_extra as usize {
            projects.pop();
            projects.reverse();
            (
                Some(SeekLink::new(Seek::Before(projects[0].name.clone()))),
                Some(SeekLink::new(Seek::After(projects[projects.len() - 1].name.clone()))),
                projects
            )
        }
        else if projects.is_empty() {
            (
                None,
                Some(SeekLink::new(Seek::Start)),
                projects
            )
        }
        else {
            projects.reverse();
            (
                None,
                Some(SeekLink::new(Seek::After(projects[projects.len() - 1].name.clone()))),
                projects
            )
        }
    )
}

fn split_title_sort_key(title: &str) -> (&str, Option<&str>) {
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

    use once_cell::sync::Lazy;

    const NOW: &str = "2023-11-12T15:50:06.419538067+00:00";

    static NOW_DT: Lazy<DateTime<Utc>> = Lazy::new(|| {
        DateTime::parse_from_rfc3339(NOW)
            .unwrap()
            .with_timezone(&Utc)
    });

    fn fake_now() -> DateTime<Utc> {
        *NOW_DT
    }

    fn make_core(pool: Pool, now: fn() -> DateTime<Utc>) -> ProdCore {
        ProdCore {
            db: pool,
            now
        }
    }

    fn fake_project_summary(name: String) -> ProjectSummary {
        ProjectSummary {
            name,
            description: "".into(),
            revision: 1,
            created_at: "".into(),
            modified_at: "".into(),
            tags: vec![],
            game: GameData {
                title: "".into(),
                title_sort_key: "".into(),
                publisher: "".into(),
                year: "".into()
            }
        }
    }

    #[sqlx::test(fixtures("ten_projects"))]
    async fn get_projects_start_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let projects: Vec<ProjectSummary> = "abcde".chars()
            .map(|c| fake_project_summary(c.into()))
            .collect();

        let prev_page = None;
        let next_page = Some(SeekLink::new(Seek::After("e".into())));

        let lp = Seek::Start;
        let limit = Limit::new(5).unwrap();

        assert_eq!(
            core.get_projects(lp, limit).await.unwrap(),
            Projects {
                projects,
                meta: Pagination {
                    prev_page,
                    next_page,
                    total: 10
                }
            }
        );
    }

    #[sqlx::test(fixtures("ten_projects"))]
    async fn get_projects_after_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let all_projects: Vec<ProjectSummary> = "abcdefghij".chars()
            .map(|c| fake_project_summary(c.into()))
            .collect();

        let lim = 5;

        // walk the limit window across the projects
        for i in 0..all_projects.len() {
            let projects: Vec<ProjectSummary> = all_projects.iter()
                .skip(i + 1)
                .take(lim)
                .cloned()
                .collect();

            let prev_page = if i == all_projects.len() - 1 {
                Some(SeekLink::new(Seek::End))
            }
            else {
                projects
                    .first()
                    .map(|p| SeekLink::new(Seek::Before(p.name.clone())))
            };

            let next_page = if i + lim + 1 >= all_projects.len() {
                None
            }
            else {
                projects
                    .last()
                    .map(|p| SeekLink::new(Seek::After(p.name.clone())))
            };

            let lp = Seek::After(all_projects[i].name.clone());
            let limit = Limit::new(lim as u8).unwrap();

            assert_eq!(
                core.get_projects(lp, limit).await.unwrap(),
                Projects {
                    projects,
                    meta: Pagination {
                        prev_page,
                        next_page,
                        total: 10
                    }
                }
            );
        }
    }

    #[sqlx::test(fixtures("ten_projects"))]
    async fn get_projects_before_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let all_projects: Vec<ProjectSummary> = "abcdefghij".chars()
            .map(|c| fake_project_summary(c.into()))
            .collect();

        let lim = 5;

        // walk the limit window across the projects
        for i in 0..all_projects.len() {
            let projects: Vec<ProjectSummary> = all_projects.iter()
                .skip(i.saturating_sub(lim))
                .take(i - i.saturating_sub(lim))
                .cloned()
                .collect();

            let prev_page = if i < lim + 1 {
                None
            }
            else {
                projects
                    .first()
                    .map(|p| SeekLink::new(Seek::Before(p.name.clone())))
            };

            let next_page = if i == 0 {
                Some(SeekLink::new(Seek::Start))
            }
            else {
                projects
                    .last()
                    .map(|p| SeekLink::new(Seek::After(p.name.clone())))
            };

            let lp = Seek::Before(all_projects[i].name.clone());
            let limit = Limit::new(lim as u8).unwrap();

            assert_eq!(
                core.get_projects(lp, limit).await.unwrap(),
                Projects {
                    projects,
                    meta: Pagination {
                        prev_page,
                        next_page,
                        total: 10
                    }
                }
            );
        }
    }

    #[sqlx::test(fixtures("ten_projects"))]
    async fn get_projects_end_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let projects: Vec<ProjectSummary> = "fghij".chars()
            .map(|c| fake_project_summary(c.into()))
            .collect();

        let prev_page = Some(SeekLink::new(Seek::Before("f".into())));
        let next_page = None;

        let lp = Seek::End;
        let limit = Limit::new(5).unwrap();

        assert_eq!(
            core.get_projects(lp, limit).await.unwrap(),
            Projects {
                projects,
                 meta: Pagination {
                    prev_page,
                    next_page,
                    total: 10
                }
            }
        );
    }

    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn get_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project(42).await.unwrap(),
            ProjectData {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 1,
                created_at: NOW.into(),
                modified_at: NOW.into(),
                tags: Vec::new(),
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn get_project_revision_ok_current(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project_revision(42, 1).await.unwrap(),
            ProjectData {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 1,
                created_at: NOW.into(),
                modified_at: NOW.into(),
                tags: Vec::new(),
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![]
            }
        );
    }

// TODO: need to show pacakges as they were?
    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn get_project_revision_ok_old(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project_revision(6, 1).await.unwrap(),
            ProjectData {
                name: "a_game".into(),
                description: "Another game".into(),
                revision: 1,
                created_at: "2019-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2019-11-12T15:50:06.419538067+00:00".into(),
                tags: Vec::new(),
                game: GameData {
                    title: "Some Otter Game".into(),
                    title_sort_key: "Some Otter Game".into(),
                    publisher: "Otters!".into(),
                    year: "1993".into()
                },
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "users"))]
    async fn create_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let user = User("bob".into());
        let proj = Project("newproj".into());
        let data = ProjectData {
            name: proj.0.clone(),
            description: "A New Game".into(),
            revision: 1,
            created_at: NOW.into(),
            modified_at: NOW.into(),
            tags: Vec::new(),
            game: GameData {
                title: "Some New Game".into(),
                title_sort_key: "Some New Game".into(),
                publisher: "XYZ Games".into(),
                year: "1999".into()
            },
            owners: vec!["bob".into()],
            packages: vec![]
        };

        let cdata = ProjectDataPut {
            description: data.description.clone(),
            tags: vec![],
            game: GameData {
                title: data.game.title.clone(),
                title_sort_key: data.game.title_sort_key.clone(),
                publisher: data.game.publisher.clone(),
                year: data.game.year.clone()
            }
        };

        core.create_project(&user, &proj.0, &cdata).await.unwrap();
        let proj_id = core.get_project_id(&proj).await.unwrap();
        assert_eq!(core.get_project(proj_id.0).await.unwrap(), data);
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn update_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let proj = Project("test_game".into());
        let new_data = ProjectData {
            name: proj.0.clone(),
            description: "new description".into(),
            revision: 2,
            created_at: NOW.into(),
            modified_at: NOW.into(),
            tags: Vec::new(),
            game: GameData {
                title: "Some New Game".into(),
                title_sort_key: "Some New Game".into(),
                publisher: "XYZ Games".into(),
                year: "1999".into()
            },
            owners: vec!["bob".into()],
            packages: vec![]
        };

        let cdata = ProjectDataPut {
            description: new_data.description.clone(),
            tags: vec![],
            game: GameData {
                title: new_data.game.title.clone(),
                title_sort_key: new_data.game.title_sort_key.clone(),
                publisher: new_data.game.publisher.clone(),
                year: new_data.game.year.clone()
            }
        };

        let proj_id = core.get_project_id(&proj).await.unwrap();
        let old_data = core.get_project(proj_id.0).await.unwrap();
        core.update_project(42, &cdata).await.unwrap();
        // project has new data
        assert_eq!(core.get_project(proj_id.0).await.unwrap(), new_data);
        // old data is kept as a revision
        assert_eq!(
            core.get_project_revision(proj_id.0, 1).await.unwrap(),
            old_data
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_package(42, 1).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_version_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_package_version(42, 1, "1.2.3").await.unwrap(),
            "https://example.com/a_package-1.2.3"
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_version_malformed(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_package_version(42, 1, "xyzzy").await.unwrap_err(),
            AppError::MalformedVersion
        );
    }

    #[sqlx::test(fixtures("projects", "packages"))]
    async fn get_package_version_not_a_version(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_package_version(42, 1, "1.0.0").await.unwrap_err(),
            AppError::NotAVersion
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(core.user_is_owner(&User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(!core.user_is_owner(&User("alice".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn add_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("alice".into())] };
        core.add_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into())
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "two_owners"))]
    async fn remove_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("bob".into())] };
        core.remove_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec![User("alice".into())] }
        );
    }

    #[sqlx::test(fixtures("projects", "one_owner"))]
    async fn remove_owners_fail_if_last(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("bob".into())] };
        assert_eq!(
            core.remove_owners(&users, 1).await.unwrap_err(),
            AppError::CannotRemoveLastOwner
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
    async fn get_players_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into())
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
    async fn add_player_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        core.add_player(&User("chuck".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users {
                users: vec![
                    User("alice".into()),
                    User("bob".into()),
                    User("chuck".into())
                ]
            }
        );
    }

    #[sqlx::test(fixtures("projects", "players"))]
    async fn remove_player_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        core.remove_player(&User("bob".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users { users: vec![User("alice".into())] }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_readme(42).await.unwrap(),
            Readme { text: "third try".into() }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_revision_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_readme_revision(42, 2).await.unwrap(),
            Readme { text: "second try".into() }
        );
    }

    #[sqlx::test(fixtures("projects", "readme"))]
    async fn get_readme_revision_bad(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_readme_revision(42, 4).await.unwrap_err(),
            AppError::NotARevision
        );
    }
}
