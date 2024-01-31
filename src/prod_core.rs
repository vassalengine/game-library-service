use axum::async_trait;
use chrono::{DateTime, Utc};
use futures_util::future::try_join_all;
use std::future::Future;

use crate::{
    core::Core,
    db::{DatabaseClient, PackageRow, ProjectRow, ProjectSummaryRow, ReleaseRow},
    errors::AppError,
    model::{GameData, Owner, PackageData, Project, ProjectData, ProjectDataPatch, ProjectDataPost, ProjectID, Projects, ProjectSummary, ReleaseData, User, Users},
    pagination::{Anchor, Limit, SortBy, Pagination, Seek, SeekLink},
    params::ProjectsParams,
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
        params: ProjectsParams
    ) -> Result<Projects, AppError>
    {
        let ProjectsParams { seek, limit } = params;
        let (prev, next, projects) = self.get_projects_from(seek, limit).await?;

        Ok(
            Projects {
                projects,
                meta: Pagination {
                    prev_page: prev.map(SeekLink::new),
                    next_page: next.map(SeekLink::new),
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
        proj_data: &ProjectDataPost
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
        owner: &Owner,
        proj_id: i64,
        proj_data: &ProjectDataPatch
    ) -> Result<(), AppError>
    {
        let now = (self.now)().to_rfc3339();
        self.db.update_project(owner, proj_id, proj_data, &now).await
    }

    async fn get_project_revision(
        &self,
        proj_id: i64,
        revision: i64
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

    async fn get_release(
        &self,
        _proj_id: i64,
        pkg_id: i64
    ) -> Result<String, AppError>
    {
        self.db.get_package_url(pkg_id).await
    }

    async fn get_release_version(
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

    async fn get_image(
        &self,
        proj_id: i64,
        img_name: &str
    ) -> Result<String, AppError>
    {
        self.db.get_image_url(proj_id, img_name).await
    }
}

impl<C: DatabaseClient + Send + Sync> ProdCore<C>  {
    async fn make_version_data(
        &self,
        rr: ReleaseRow
    ) -> Result<ReleaseData, AppError>
    {
        let authors = self.db.get_authors(rr.release_id)
            .await?
            .users
            .into_iter()
            .map(|u| u.0)
            .collect();

        Ok(
            ReleaseData {
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
        let releases = try_join_all(
            get_release_rows(self, pr.package_id)
                .await?
                .into_iter()
                .map(|vr| self.make_version_data(vr))
        ).await?;

        Ok(
            PackageData {
                name: pr.name,
                description: "".into(),
                releases
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
                readme: proj_row.readme,
                image: proj_row.image,
                owners,
                packages
            }
        )
    }

    async fn get_projects_from(
        &self,
        seek: Seek,
        limit: Limit
    ) -> Result<(Option<Seek>, Option<Seek>, Vec<ProjectSummary>), AppError>
    {
        // unpack the seek
        let Seek { sort_by, dir, anchor } = seek;

        // try to get one extra so we can tell if we're at an endpoint
        let limit_extra = limit.get() as u32 + 1;

        // get the window, seeking forward
        let mut projects = match anchor {
            Anchor::Start =>
                self.db.get_projects_end_window(
                    sort_by,
                    dir,
                    limit_extra
                ).await,
            Anchor::After(ref field, id) =>
                self.db.get_projects_mid_window(
                    sort_by,
                    dir,
                    field,
                    id,
                    limit_extra
                ).await,
            Anchor::Before(ref field, id) =>
                self.db.get_projects_mid_window(
                    sort_by,
                    dir.rev(),
                    field,
                    id,
                    limit_extra
                ).await,
            Anchor::StartQuery(ref query) =>
                self.db.get_projects_query_end_window(
                    query,
                    dir,
                    limit_extra
                ).await,
            Anchor::AfterQuery(ref query, rank, id) =>
                self.db.get_projects_query_mid_window(
                    query,
                    dir,
                    rank,
                    id,
                    limit_extra
                ).await,
            Anchor::BeforeQuery(ref query, rank, id) =>
                self.db.get_projects_query_mid_window(
                    query,
                    dir.rev(),
                    rank,
                    id,
                    limit_extra
                ).await
        }?;

        // make the next link
        let next = if projects.len() == limit_extra as usize {
            // there are more pages in the forward direction

            // remove the "extra" item which proves we are not at the end
            projects.pop();

            // the next page is after the last item
            let last = match anchor {
                Anchor::Before(_, _) => projects.first(),
                _ => projects.last()
            }.expect("element must exist");

            Some(
                Seek {
                    anchor: Anchor::After(
                        last.sort_field(sort_by).into(),
                        last.project_id as u32
                    ),
                    sort_by,
                    dir
                }
            )
        }
        else {
            // there are no pages in the forward direction
            None
        };

        // make the prev link
        let prev = if projects.is_empty() {
            None
        }
        else {
            match anchor {
                Anchor::Start => None,
                _ => {
                    // the previous page is before the first item
                    let first = match anchor {
                        Anchor::Before(_, _) => projects.last(),
                        _ => projects.first()
                    }.expect("element must exist");

                    Some(
                        Seek {
                            anchor: Anchor::Before(
                                first.sort_field(sort_by).into(),
                                first.project_id as u32
                            ),
                            sort_by,
                            dir
                        }
                    )
                }
            }
        };

        let pi = projects.into_iter().map(ProjectSummary::from);
        let psums = match anchor {
            Anchor::Before(_, _) => pi.rev().collect(),
            _ => pi.collect()
        }; 

        Ok((prev, next, psums)) 
    }
}

impl ProjectSummaryRow {
    fn sort_field(&self, sort_by: SortBy) -> &str {
        match sort_by {
            SortBy::ProjectName => &self.name,
            SortBy::GameTitle => &self.game_title_sort,
            SortBy::ModificationTime => &self.modified_at,
            SortBy::CreationTime => &self.created_at
        }
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

    use crate::{
        model::GameDataPatch,
        pagination::Direction,
        sqlite::{Pool, SqlxDatabaseClient}
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

    fn fake_project_summary(name: &str) -> ProjectSummary {
        ProjectSummary {
            name: name.into(),
            description: "".into(),
            revision: 1,
            created_at: "".into(),
            modified_at: format!(
                "2024-{:02}-01T13:51:50+00:00",
                name.as_bytes()[0] - b'a' + 1
            ),
            tags: vec![],
            game: GameData {
                title: "".into(),
                title_sort_key: "".into(),
                publisher: "".into(),
                year: "".into()
            }
        }
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_start_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::Start
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("a"),
                fake_project_summary("b"),
                fake_project_summary("c")
            ]
        );

        assert_eq!(prev, None);

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("c".into(), 3),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_end_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::Start
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("j"),
                fake_project_summary("i"),
                fake_project_summary("h")
            ]
        );

        assert_eq!(prev, None);

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("h".into(), 8),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_after_asc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::After("a".into(), 1)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("b"),
                fake_project_summary("c"),
                fake_project_summary("d")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("b".into(), 2),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("d".into(), 4),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_after_desc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::After("h".into(), 8)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("g"),
                fake_project_summary("f"),
                fake_project_summary("e")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("g".into(), 7),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("e".into(), 5),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_before_asc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::Before("e".into(), 5)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("b"),
                fake_project_summary("c"),
                fake_project_summary("d")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("b".into(), 2),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("d".into(), 4),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_before_desc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::Before("e".into(), 5)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("h"),
                fake_project_summary("g"),
                fake_project_summary("f")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("h".into(), 8),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("f".into(), 6),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_start_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Descending,
                anchor: Anchor::Start
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("j"),
                fake_project_summary("i"),
                fake_project_summary("h")
            ]
        );

        assert_eq!(prev, None);

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("2024-08-01T13:51:50+00:00".into(), 8),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_end_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::Start
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("j"),
                fake_project_summary("i"),
                fake_project_summary("h")
            ]
        );

        assert_eq!(prev, None);

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("h".into(), 8),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_after_asc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Ascending,
                anchor: Anchor::After("2024-01-01T13:51:50+00:00".into(), 1)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("b"),
                fake_project_summary("c"),
                fake_project_summary("d")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("2024-02-01T13:51:50+00:00".into(), 2),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("2024-04-01T13:51:50+00:00".into(), 4),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_after_desc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Descending,
                anchor: Anchor::After("2024-08-01T13:51:50+00:00".into(), 8)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("g"),
                fake_project_summary("f"),
                fake_project_summary("e")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("2024-07-01T13:51:50+00:00".into(), 7),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("2024-05-01T13:51:50+00:00".into(), 5),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_before_asc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Ascending,
                anchor: Anchor::Before("2024-05-01T13:51:50+00:00".into(), 5)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("b"),
                fake_project_summary("c"),
                fake_project_summary("d")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("2024-02-01T13:51:50+00:00".into(), 2),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("2024-04-01T13:51:50+00:00".into(), 4),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_before_desc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Descending,
                anchor: Anchor::Before("2024-05-01T13:51:50+00:00".into(), 5)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("h"),
                fake_project_summary("g"),
                fake_project_summary("f")
            ]
        );

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("2024-08-01T13:51:50+00:00".into(), 8),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After("2024-06-01T13:51:50+00:00".into(), 6),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "projects", "two_owners", "packages", "authors"))]
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
                tags: vec![],
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                readme: "".into(),
                image: None,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "a_package".into(),
                        description: "".into(),
                        releases: vec![
                            ReleaseData {
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
                            ReleaseData {
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
                        releases: vec![]
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        releases: vec![
                            ReleaseData {
                                version: "0.1.0".into(),
                                filename: "c_package-0.1.0".into(),
                                url: "https://example.com/c_package-0.1.0".into(),
                                size: 123456,
                                checksum: "a8f515e9e2de99919d1a987733296aaa951a4ba2aa0f7014c510bdbd60dc0efd".into(),
                                published_at: "2023-12-15T15:56:29.180282477+00:00".into(),
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

    #[sqlx::test(fixtures("users", "projects", "two_owners", "packages", "authors"))]
    async fn get_project_revision_ok_current(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_project_revision(42, 3).await.unwrap(),
            ProjectData {
                name: "test_game".into(),
                description: "Brian's Trademarked Game of Being a Test Case".into(),
                revision: 3,
                created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
                modified_at: "2023-12-14T15:50:06.419538067+00:00".into(),
                tags: vec![],
                game: GameData {
                    title: "A Game of Tests".into(),
                    title_sort_key: "Game of Tests, A".into(),
                    publisher: "Test Game Company".into(),
                    year: "1979".into()
                },
                readme: "".into(),
                image: None,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "a_package".into(),
                        description: "".into(),
                        releases: vec![
                            ReleaseData {
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
                            ReleaseData {
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
                        releases: vec![]
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        releases: vec![]
                    }
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "two_owners", "packages"))]
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
                    year: "1978".into()
                },
                readme: "".into(),
                image: None,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "b_package".into(),
                        description: "".into(),
                        releases: vec![]
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        releases: vec![]
                    }
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
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
            tags: vec![],
            game: GameData {
                title: "Some New Game".into(),
                title_sort_key: "Some New Game".into(),
                publisher: "XYZ Games".into(),
                year: "1999".into()
            },
            readme: "".into(),
            image: None,
            owners: vec!["bob".into()],
            packages: vec![]
        };

        let cdata = ProjectDataPost {
            description: data.description.clone(),
            tags: vec![],
            game: GameData {
                title: data.game.title.clone(),
                title_sort_key: data.game.title_sort_key.clone(),
                publisher: data.game.publisher.clone(),
                year: data.game.year.clone()
            },
            readme: "".into(),
            image: None
        };

        core.create_project(&user, &proj.0, &cdata).await.unwrap();
        let proj_id = core.get_project_id(&proj).await.unwrap();
        assert_eq!(core.get_project(proj_id.0).await.unwrap(), data);
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn update_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let proj = Project("test_game".into());
        let new_data = ProjectData {
            name: proj.0.clone(),
            description: "new description".into(),
            revision: 4,
            created_at: "2023-11-12T15:50:06.419538067+00:00".into(),
            modified_at: NOW.into(),
            tags: vec![],
            game: GameData {
                title: "Some New Game".into(),
                title_sort_key: "Some New Game".into(),
                publisher: "XYZ Games".into(),
                year: "1999".into()
            },
            readme: "".into(),
            image: None,
            owners: vec!["bob".into()],
            packages: vec![]
        };

        let cdata = ProjectDataPatch {
            description: Some(new_data.description.clone()),
            tags: Some(vec![]),
            game: GameDataPatch {
                title: Some(new_data.game.title.clone()),
                title_sort_key: Some(new_data.game.title_sort_key.clone()),
                publisher: Some(new_data.game.publisher.clone()),
                year: Some(new_data.game.year.clone())
            },
            readme: Some("".into()),
            image: None
        };

        let proj_id = core.get_project_id(&proj).await.unwrap();
        let old_data = core.get_project(proj_id.0).await.unwrap();
        core.update_project(&Owner("bob".into()), 42, &cdata).await.unwrap();
        // project has new data
        assert_eq!(core.get_project(proj_id.0).await.unwrap(), new_data);
        // old data is kept as a revision
        assert_eq!(
            core.get_project_revision(proj_id.0, 3).await.unwrap(),
            old_data
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_release(42, 1).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_version_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let version = "1.2.3".parse::<Version>().unwrap();
        assert_eq!(
            core.get_release_version(42, 1, &version).await.unwrap(),
            "https://example.com/a_package-1.2.3"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_version_not_a_version(pool: Pool) {
        let core = make_core(pool, fake_now);
        let version = "1.0.0".parse::<Version>().unwrap();
        assert_eq!(
            core.get_release_version(42, 1, &version).await.unwrap_err(),
            AppError::NotAVersion
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec![User("bob".into())] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(core.user_is_owner(&User("bob".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(!core.user_is_owner(&User("alice".into()), 42).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
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

    #[sqlx::test(fixtures("users", "projects", "two_owners"))]
    async fn remove_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("bob".into())] };
        core.remove_owners(&users, 42).await.unwrap();
        assert_eq!(
            core.get_owners(42).await.unwrap(),
            Users { users: vec![User("alice".into())] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn remove_owners_fail_if_last(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec![User("bob".into())] };
        assert_eq!(
            core.remove_owners(&users, 1).await.unwrap_err(),
            AppError::CannotRemoveLastOwner
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
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

    #[sqlx::test(fixtures("users", "projects", "players"))]
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

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn remove_player_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        core.remove_player(&User("bob".into()), 42).await.unwrap();
        assert_eq!(
            core.get_players(42).await.unwrap(),
            Users { users: vec![User("alice".into())] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(42, "img.png").await.unwrap(),
            "https://example.com/images/img.png"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_not_a_project(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(1, "img.png").await.unwrap_err(),
            AppError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_not_an_image(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(42, "bogus").await.unwrap_err(),
            AppError::NotFound
        );
    }
}
