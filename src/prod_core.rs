use axum::async_trait;
use chrono::{DateTime, Utc};
use futures_util::future::try_join_all;
use std::future::Future;

use crate::{
    core::Core,
    db::{DatabaseClient, PackageRow, ProjectRow, ReleaseRow},
    errors::AppError,
    model::{GameData, PackageData, Project, ProjectData, ProjectDataPut, ProjectID, Projects, ProjectSummary, Readme, User, Users, VersionData},
    pagination::{Limit, Pagination, Seek, SeekLink},
    version::Version
};

#[derive(Clone)]
pub struct ProdCore<C: DatabaseClient> {
    pub db: C,
    pub now: fn() -> DateTime<Utc>
}

// TODO: switch proj_id to proj_name; then we will always know if the project
// exists because we have to look up the id

#[async_trait]
impl<C: DatabaseClient + Send + Sync> Core for ProdCore<C> {
    async fn get_project_id(
         &self,
        proj: &Project
    ) -> Result<ProjectID, AppError>
    {
        self.db.get_project_id(&proj.0).await
    }

    async fn get_owners(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        self.db.get_owners(proj_id).await
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        self.db.add_owners(owners, proj_id).await
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        self.db.remove_owners(owners, proj_id).await
    }

    async fn user_is_owner(
        &self,
        user: &User,
        proj_id: i64
    ) -> Result<bool, AppError>
    {
        self.db.user_is_owner(user, proj_id).await
    }

    async fn get_projects(
        &self,
        from: Seek,
        limit: Limit
    ) -> Result<Projects, AppError>
    {
        let limit = limit.get() as u32;

        let (prev_page, next_page, projects) = match from {
            Seek::Start => self.get_projects_start(limit).await?,
            Seek::After(name) => self.get_projects_after(&name, limit).await?,
            Seek::Before(name) => self.get_projects_before(&name, limit).await?,
            Seek::End => self.get_projects_end(limit).await?
        };

        Ok(
            Projects {
                projects,
                meta: Pagination {
                    prev_page,
                    next_page,
                    total: self.db.get_project_count().await?
                }
            },
        )
    }

    async fn get_project(
        &self,
        proj_id: i64
    ) -> Result<ProjectData, AppError>
    {
        self.get_project_impl(
            proj_id,
            self.db.get_project_row(proj_id).await?,
            self.db.get_packages(proj_id).await?,
            |pc, pkgid| pc.db.get_releases(pkgid)
        ).await
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
// FIXME: generate a sort key?
//        let mut proj_data = proj_data;
//        proj_data.game.title_sort_key = title_sort_key(&proj_data.game.title);
        self.db.create_project(user, proj, proj_data, &now).await
    }

    async fn update_project(
        &self,
        proj_id: i64,
        proj_data: &ProjectDataPut
    ) -> Result<(), AppError>
    {
        let now = (self.now)().to_rfc3339();
        self.db.update_project(proj_id, proj_data, &now).await
    }

    async fn get_project_revision(
        &self,
        proj_id: i64,
        revision: u32
    ) -> Result<ProjectData, AppError>
    {
        let proj_row = self.db.get_project_row_revision(
            proj_id, revision
        ).await?;

        let mtime = proj_row.modified_at.clone();

        let package_rows = self.db.get_packages_at(
            proj_id, &mtime
        ).await?;

        self.get_project_impl(
            proj_id,
            proj_row,
            package_rows,
            |pc, pkgid| pc.db.get_releases_at(pkgid, &mtime)
        ).await
    }

    async fn get_package(
        &self,
        _proj_id: i64,
        pkg_id: i64
    ) -> Result<String, AppError>
    {
        self.db.get_package_url(pkg_id).await
    }

    async fn get_release(
        &self,
        _proj_id: i64,
        pkg_id: i64,
        version: &Version
    ) -> Result<String, AppError>
    {
        self.db.get_release_url(pkg_id, version).await
    }

    async fn get_players(
        &self,
        proj_id: i64
    ) -> Result<Users, AppError>
    {
        self.db.get_players(proj_id).await
    }

    async fn add_player(
        &self,
        player: &User,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        self.db.add_player(player, proj_id).await
    }

    async fn remove_player(
        &self,
        player: &User,
        proj_id: i64
    ) -> Result<(), AppError>
    {
        self.db.remove_player(player, proj_id).await
    }

    async fn get_readme(
        &self,
        readme_id: i64
    ) -> Result<Readme, AppError>
    {
        self.db.get_readme(readme_id).await
    }
}

impl<C: DatabaseClient + Send + Sync> ProdCore<C>  {
    async fn make_version_data(
        &self,
        rr: ReleaseRow
    ) -> Result<VersionData, AppError>
    {
        let authors = self.db.get_authors(rr.release_id)
            .await?
            .users
            .into_iter()
            .map(|u| u.0)
            .collect();

        Ok(
            VersionData {
                version: rr.version,
                filename: rr.filename,
                url: rr.url,
                size: rr.size,
                checksum: rr.checksum,
                published_at: rr.published_at,
                published_by: rr.published_by,
                requires: "".into(),
                authors
            }
        )
    }

    async fn make_package_data<'s, F, R>(
        &'s self,
        pr: PackageRow,
        get_release_rows: &F
    ) -> Result<PackageData, AppError>
    where
        F: Fn(&'s Self, i64) -> R,
        R: Future<Output = Result<Vec<ReleaseRow>, AppError>>
    {
        let versions = try_join_all(
            get_release_rows(self, pr.package_id)
                .await?
                .into_iter()
                .map(|vr| self.make_version_data(vr))
        ).await?;

        Ok(
            PackageData {
                name: pr.name,
                description: "".into(),
                versions
            }
        )
    }

    async fn get_project_impl<'s, F, R>(
        &'s self,
        proj_id: i64,
        proj_row: ProjectRow,
        package_rows: Vec<PackageRow>,
        get_release_rows: F
    ) -> Result<ProjectData, AppError>
    where
        F: Fn(&'s Self, i64) -> R,
        R: Future<Output = Result<Vec<ReleaseRow>, AppError>>
    {
        let owners = self.get_owners(proj_id)
            .await?
            .users
            .into_iter()
            .map(|u| u.0)
            .collect();

        let packages = try_join_all(
            package_rows
                .into_iter()
                .map(|pr| self.make_package_data(pr, &get_release_rows))
        ).await?;

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
                readme_id: proj_row.readme_id,
                owners,
                packages
            }
        )
    }

    async fn get_projects_start(
        &self,
        limit: u32
    ) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
    {
        // try to get one extra so we can tell if we're at an endpoint
        let limit_extra = limit + 1;

        let mut projects = self.db.get_projects_start_window(limit_extra).await?;

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
        &self,
        limit: u32
    ) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
    {
        // try to get one extra so we can tell if we're at an endpoint
        let limit_extra = limit + 1;

        let mut projects = self.db.get_projects_end_window(limit_extra).await?;

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
        &self,
        name: &str,
        limit: u32
    ) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
    {
        // try to get one extra so we can tell if we're at an endpoint
        let limit_extra = limit + 1;

        let mut projects = self.db.get_projects_after_window(name, limit_extra)
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
        &self,
        name: &str,
        limit: u32
    ) -> Result<(Option<SeekLink>, Option<SeekLink>, Vec<ProjectSummary>), AppError>
    {
        // try to get one extra so we can tell if we're at an endpoint
        let limit_extra = limit + 1;

        let mut projects = self.db.get_projects_before_window(name, limit_extra)
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

    use crate::sqlite::{
        Pool, SqlxDatabaseClient
    };

    const NOW: &str = "2023-11-12T15:50:06.419538067+00:00";

    static NOW_DT: Lazy<DateTime<Utc>> = Lazy::new(|| {
        DateTime::parse_from_rfc3339(NOW)
            .unwrap()
            .with_timezone(&Utc)
    });

    fn fake_now() -> DateTime<Utc> {
        *NOW_DT
    }

    fn make_core(
        pool: Pool,
        now: fn() -> DateTime<Utc>
    ) -> ProdCore<SqlxDatabaseClient<sqlx::sqlite::Sqlite>>
    {
        ProdCore {
            db: SqlxDatabaseClient(pool),
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

    #[sqlx::test(fixtures("readmes", "ten_projects"))]
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

    #[sqlx::test(fixtures("readmes", "ten_projects"))]
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

    #[sqlx::test(fixtures("readmes", "ten_projects"))]
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

    #[sqlx::test(fixtures("readmes", "ten_projects"))]
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

    #[sqlx::test(fixtures("readmes", "projects", "users", "two_owners", "packages", "authors"))]
    async fn get_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project(42).await.unwrap(),
            ProjectData {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 3,
                created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2023-12-14T15:50:06.419538067+00:00".into(),
                tags: Vec::new(),
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                readme_id: 8,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "a_package".into(),
                        description: "".into(),
                        versions: vec![
                            VersionData {
                                version: "1.2.4".into(),
                                filename: "a_package-1.2.4".into(),
                                url: "https://example.com/a_package-1.2.4".into(),
                                size: 5678,
                                checksum: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
                                published_at: "2023-12-10T15:56:29.180282477+00:00".into(),
                                published_by: "alice".into(),
                                requires: "".into(),
                                authors: vec!["alice".into(), "bob".into()]
                            },
                            VersionData {
                                version: "1.2.3".into(),
                                filename: "a_package-1.2.3".into(),
                                url: "https://example.com/a_package-1.2.3".into(),
                                size: 1234,
                                checksum: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
                                published_at: "2023-12-09T15:56:29.180282477+00:00".into(),
                                published_by: "bob".into(),
                                requires: "".into(),
                                authors: vec!["alice".into()]
                            }
                        ]
                    },
                    PackageData {
                        name: "b_package".into(),
                        description: "".into(),
                        versions: vec![]
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        versions: vec![
                            VersionData {
                                version: "0.1.0".into(),
                                filename: "c_package-0.1.0".into(),
                                url: "https://example.com/c_package-0.1.0".into(),
                                size: 123456,
                                checksum: "a8f515e9e2de99919d1a987733296aaa951a4ba2aa0f7014c510bdbd60dc0efd".into(),
                                published_at: "2023-12-13T15:56:29.180282477+00:00".into(),
                                published_by: "chuck".into(),
                                requires: "".into(),
                                authors: vec![]
                            }
                        ]
                    }
                ]
            }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "two_owners", "packages", "authors"))]
    async fn get_project_revision_ok_current(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project_revision(42, 2).await.unwrap(),
            ProjectData {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 2,
                created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2023-12-12T15:50:06.419538067+00:00".into(),
                tags: Vec::new(),
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                readme_id: 8,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "a_package".into(),
                        description: "".into(),
                        versions: vec![
                            VersionData {
                                version: "1.2.4".into(),
                                filename: "a_package-1.2.4".into(),
                                url: "https://example.com/a_package-1.2.4".into(),
                                size: 5678,
                                checksum: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
                                published_at: "2023-12-10T15:56:29.180282477+00:00".into(),
                                published_by: "alice".into(),
                                requires: "".into(),
                                authors: vec!["alice".into(), "bob".into()]
                            },
                            VersionData {
                                version: "1.2.3".into(),
                                filename: "a_package-1.2.3".into(),
                                url: "https://example.com/a_package-1.2.3".into(),
                                size: 1234,
                                checksum: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
                                published_at: "2023-12-09T15:56:29.180282477+00:00".into(),
                                published_by: "bob".into(),
                                requires: "".into(),
                                authors: vec!["alice".into()]
                            }
                        ]
                    },
                    PackageData {
                        name: "b_package".into(),
                        description: "".into(),
                        versions: vec![]
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        versions: vec![]
                    }
                ]
            }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "two_owners", "packages"))]
    async fn get_project_revision_ok_old(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project_revision(42, 1).await.unwrap(),
            ProjectData {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 1,
                created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                tags: vec![],
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                readme_id: 8,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "b_package".into(),
                        description: "".into(),
                        versions: vec![]
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        versions: vec![]
                    }
                ]
            }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "packages"))]
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
            readme_id: 0,
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

    #[sqlx::test(fixtures("readmes", "projects", "users", "one_owner"))]
    async fn update_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let proj = Project("test_game".into());
        let new_data = ProjectData {
            name: proj.0.clone(),
            description: "new description".into(),
            revision: 4,
            created_at: NOW.into(),
            modified_at: NOW.into(),
            tags: Vec::new(),
            game: GameData {
                title: "Some New Game".into(),
                title_sort_key: "Some New Game".into(),
                publisher: "XYZ Games".into(),
                year: "1999".into()
            },
            readme_id: 8,
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
            core.get_project_revision(proj_id.0, 3).await.unwrap(),
            old_data
        );
    }

    #[sqlx::test(fixtures("users", "readmes", "projects", "packages"))]
    async fn get_package_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_package(42, 1).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("users", "readmes", "projects", "packages"))]
    async fn get_release_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let version = "1.2.3".parse::<Version>().unwrap();
        assert_eq!(
            core.get_release(42, 1, &version).await.unwrap(),
            "https://example.com/a_package-1.2.3"
        );
    }

    #[sqlx::test(fixtures("users", "readmes", "projects", "packages"))]
    async fn get_release_not_a_version(pool: Pool) {
        let core = make_core(pool, fake_now);
        let version = "1.0.0".parse::<Version>().unwrap();
        assert_eq!(
            core.get_release(42, 1, &version).await.unwrap_err(),
            AppError::NotAVersion
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(core.user_is_owner(&User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(!core.user_is_owner(&User("alice".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "one_owner"))]
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

    #[sqlx::test(fixtures("readmes", "projects", "users", "two_owners"))]
    async fn remove_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("bob".into())] };
        core.remove_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec![User("alice".into())] }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "one_owner"))]
    async fn remove_owners_fail_if_last(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("bob".into())] };
        assert_eq!(
            core.remove_owners(&users, 1).await.unwrap_err(),
            AppError::CannotRemoveLastOwner
        );
    }

    #[sqlx::test(fixtures("readmes", "projects", "users", "players"))]
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

    #[sqlx::test(fixtures("readmes", "projects", "users", "players"))]
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

    #[sqlx::test(fixtures("readmes", "projects", "users", "players"))]
    async fn remove_player_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        core.remove_player(&User("bob".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users { users: vec![User("alice".into())] }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects"))]
    async fn get_readme_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_readme(8).await.unwrap(),
            Readme { text: "hey".into() }
        );
    }

    #[sqlx::test(fixtures("readmes", "projects"))]
    async fn get_readme_not_a_readme(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_readme(1).await.unwrap_err(),
            AppError::NotARevision
        );
    }
}
