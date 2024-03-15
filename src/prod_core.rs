use axum::async_trait;
use chrono::{DateTime, Utc};
use futures_util::future::try_join_all;
use std::future::Future;

use crate::{
    core::Core,
    db::{DatabaseClient, PackageRow, ProjectRow, ProjectSummaryRow, ReleaseRow},
    errors::AppError,
    model::{GameData, Owner, Package, PackageData, PackageDataPost, ProjectData, ProjectDataPatch, ProjectDataPost, Project, Projects, ProjectSummary, ReleaseData, User, Users},
    pagination::{Anchor, Direction, Limit, SortBy, Pagination, Seek, SeekLink},
    params::ProjectsParams,
    time::nanos_to_rfc3339,
    version::Version
};

#[derive(Clone)]
pub struct ProdCore<C: DatabaseClient> {
    pub db: C,
    pub now: fn() -> DateTime<Utc>
}

// TODO: Push User, Owner, Project all the way inward

#[async_trait]
impl<C: DatabaseClient + Send + Sync> Core for ProdCore<C> {
    async fn get_user_id(
         &self,
        username: &str
    ) -> Result<User, AppError>
    {
        Ok(self.db.get_user_id(&username).await?)
    }

    async fn get_project_id(
         &self,
        proj: &str
    ) -> Result<Project, AppError>
    {
        self.db.get_project_id(proj).await
    }

    async fn get_owners(
        &self,
        proj: Project
    ) -> Result<Users, AppError>
    {
        self.db.get_owners(proj).await
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), AppError>
    {
        self.db.add_owners(owners, proj).await
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), AppError>
    {
        self.db.remove_owners(owners, proj).await
    }

    async fn user_is_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<bool, AppError>
    {
        self.db.user_is_owner(user, proj).await
    }

    async fn get_projects(
        &self,
        params: ProjectsParams
    ) -> Result<Projects, AppError>
    {
        let ProjectsParams { seek, limit } = params;
        let (prev, next, projects, total) = self.get_projects_from(
            seek, limit.unwrap_or_default()
        ).await?;

        let prev_page = match prev {
            Some(prev) => Some(SeekLink::new(&prev, limit)?),
            None => None
        };

        let next_page = match next {
            Some(next) => Some(SeekLink::new(&next, limit)?),
            None => None
        };

        Ok(
            Projects {
                projects,
                meta: Pagination {
                    prev_page,
                    next_page,
                    total
                }
            },
        )
    }

    async fn get_project(
        &self,
        proj: Project
    ) -> Result<ProjectData, AppError>
    {
        self.get_project_impl(
            proj,
            self.db.get_project_row(proj).await?,
            self.db.get_packages(proj).await?,
            |pc, pkg| pc.db.get_releases(pkg)
        ).await
    }

// TODO: require project names to match [A-Za-z0-9][A-Za-z0-9_-]{,63}?
// TODO: maybe also compare case-insensitively and equate - and _?
// TODO: length limits on strings
// TODO: require package names to match [A-Za-z0-9][A-Za-z0-9_-]{,63}?
// TODO: packages might need display names?

    async fn create_project(
        &self,
        user: User,
        proj: &str,
        proj_data: &ProjectDataPost
    ) -> Result<(), AppError>
    {
        let now = (self.now)()
            .timestamp_nanos_opt()
            .ok_or(AppError::InternalError)?;
// FIXME: generate a sort key?
//        let mut proj_data = proj_data;
//        proj_data.game.title_sort_key = title_sort_key(&proj_data.game.title);
        self.db.create_project(user, proj, proj_data, now).await
    }

    async fn update_project(
        &self,
        owner: Owner,
        proj: Project,
        proj_data: &ProjectDataPatch
    ) -> Result<(), AppError>
    {
        let now = (self.now)()
            .timestamp_nanos_opt()
            .ok_or(AppError::InternalError)?;
        self.db.update_project(owner, proj, proj_data, now).await
    }

    async fn get_project_revision(
        &self,
        proj: Project,
        revision: i64
    ) -> Result<ProjectData, AppError>
    {
        let proj_row = self.db.get_project_row_revision(proj, revision)
            .await?;

        let mtime = proj_row.modified_at;

        let package_rows = self.db.get_packages_at(proj, mtime).await?;

        self.get_project_impl(
            proj,
            proj_row,
            package_rows,
            |pc, pkg| pc.db.get_releases_at(pkg, mtime)
        ).await
    }

    async fn create_package(
        &self,
        owner: Owner,
        proj: Project,
        pkg: &str,
        pkg_data: &PackageDataPost
    ) -> Result<(), AppError>
    {
        let now = (self.now)()
            .timestamp_nanos_opt()
            .ok_or(AppError::InternalError)?;
        self.db.create_package(owner, proj, pkg, pkg_data, now).await
    }

    async fn get_release(
        &self,
        _proj: Project,
        pkg: Package
    ) -> Result<String, AppError>
    {
        self.db.get_package_url(pkg).await
    }

    async fn get_release_version(
        &self,
        _proj: Project,
        pkg: Package,
        version: &Version
    ) -> Result<String, AppError>
    {
        self.db.get_release_url(pkg, version).await
    }

    async fn get_players(
        &self,
        proj: Project
    ) -> Result<Users, AppError>
    {
        self.db.get_players(proj).await
    }

    async fn add_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), AppError>
    {
        self.db.add_player(player, proj).await
    }

    async fn remove_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), AppError>
    {
        self.db.remove_player(player, proj).await
    }

    async fn get_image(
        &self,
        proj: Project,
        img_name: &str
    ) -> Result<String, AppError>
    {
        self.db.get_image_url(proj, img_name).await
    }

    async fn get_image_revision(
        &self,
        proj: Project,
        revision: i64,
        img_name: &str
    ) -> Result<String, AppError>
    {
        // TODO: this could be a join
        let proj_row = self.db.get_project_row_revision(proj, revision).await?;

        let mtime = proj_row.modified_at;

        self.db.get_image_url_at(proj, img_name, mtime).await
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
            .users;

        Ok(
            ReleaseData {
                version: rr.version,
                filename: rr.filename,
                url: rr.url,
                size: rr.size,
                checksum: rr.checksum,
                published_at: nanos_to_rfc3339(rr.published_at)?,
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
        F: Fn(&'s Self, Package) -> R,
        R: Future<Output = Result<Vec<ReleaseRow>, AppError>>
    {
        let releases = try_join_all(
            get_release_rows(self, Package(pr.package_id))
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
        proj: Project,
        proj_row: ProjectRow,
        package_rows: Vec<PackageRow>,
        get_release_rows: F
    ) -> Result<ProjectData, AppError>
    where
        F: Fn(&'s Self, Package) -> R,
        R: Future<Output = Result<Vec<ReleaseRow>, AppError>>
    {
        let owners = self.get_owners(proj)
            .await?
            .users;

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
                created_at: nanos_to_rfc3339(proj_row.created_at)?,
                modified_at: nanos_to_rfc3339(proj_row.modified_at)?,
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

    async fn get_projects_window(
        &self,
        anchor: &Anchor,
        sort_by: SortBy,
        dir: Direction,
        limit_extra: u32
    ) -> Result<Vec<ProjectSummaryRow>, AppError>
    {
        match anchor {
            Anchor::Start =>
                self.db.get_projects_end_window(
                    sort_by,
                    dir,
                    limit_extra
                ),
            Anchor::After(field, id) =>
                self.db.get_projects_mid_window(
                    sort_by,
                    dir,
                    field,
                    *id,
                    limit_extra
                ),
            Anchor::Before(field, id) =>
                self.db.get_projects_mid_window(
                    sort_by,
                    dir.rev(),
                    field,
                    *id,
                    limit_extra
                ),
            Anchor::StartQuery(query) =>
                self.db.get_projects_query_end_window(
                    query,
                    sort_by,
                    dir,
                    limit_extra
                ),
            Anchor::AfterQuery(query, field, id) =>
                self.db.get_projects_query_mid_window(
                    query,
                    sort_by,
                    dir,
                    field,
                    *id,
                    limit_extra
                ),
            Anchor::BeforeQuery(query, field, id) =>
                self.db.get_projects_query_mid_window(
                    query,
                    sort_by,
                    dir.rev(),
                    field,
                    *id,
                    limit_extra
                )
        }.await
    }

    async fn get_projects_from(
        &self,
        seek: Seek,
        limit: Limit
    ) -> Result<(Option<Seek>, Option<Seek>, Vec<ProjectSummary>, i64), AppError>
    {
        // unpack the seek
        let Seek { sort_by, dir, anchor } = seek;

        // try to get one extra so we can tell if we're at an endpoint
        let limit_extra = limit.get() as u32 + 1;

        // get the window
        let mut projects = self.get_projects_window(
            &anchor,
            sort_by,
            dir,
            limit_extra
        ).await?;

        // get the prev, next links
        let (prev, next) = get_links(
            &anchor,
            sort_by,
            dir,
            limit_extra,
            &mut projects
        )?;

        // get the total number of responsive items
        let total = match anchor {
            Anchor::StartQuery(ref q) |
            Anchor::AfterQuery(ref q, _, _) |
            Anchor::BeforeQuery(ref q, _, _) =>
                self.db.get_projects_query_count(q),
            _ => self.db.get_projects_count()
        }.await?;

        // convert the rows to summaries
        let pi = projects.into_iter().map(ProjectSummary::try_from);
        let psums = match anchor {
            Anchor::Before(..) |
            Anchor::BeforeQuery(..) => pi.rev().collect::<Result<Vec<_>, _>>(),
            _ => pi.collect::<Result<Vec<_>, _>>()
        }?;

        Ok((prev, next, psums, total))
    }
}

fn get_prev_for_before(
    anchor: &Anchor,
    sort_by: SortBy,
    dir: Direction,
    limit_extra: u32,
    projects: &mut Vec<ProjectSummaryRow>
) -> Result<Option<Seek>, AppError>
{
    // make the prev link
    if projects.len() == limit_extra as usize {
        // there are more pages in the forward direction

        // remove the "extra" item which proves we are not at the end
        projects.pop();

        // the prev page is after the last item
        let last = projects.last().expect("element must exist");

        let prev_anchor = match anchor {
            Anchor::BeforeQuery(ref q, _, _) => Anchor::BeforeQuery(
                q.clone(),
                last.sort_field(sort_by)?,
                last.project_id as u32
            ),
            Anchor::Before(..) => Anchor::Before(
                last.sort_field(sort_by)?,
                last.project_id as u32
            ),
            Anchor::Start |
            Anchor::StartQuery(..) |
            Anchor::After(..) |
            Anchor::AfterQuery(..) => unreachable!()
        };

        Ok(Some(Seek { anchor: prev_anchor, sort_by, dir }))
    }
    else {
        // there are no pages in the forward direction
        Ok(None)
    }
}

fn get_next_for_before(
    anchor: &Anchor,
    sort_by: SortBy,
    dir: Direction,
    projects: &Vec<ProjectSummaryRow>
) -> Result<Option<Seek>, AppError>
{
    // make the next link
    if projects.is_empty() {
        Ok(None)
    }
    else {
        // the next page is before the first item
        let first = projects.first().expect("element must exist");

        let next_anchor = match anchor {
            Anchor::BeforeQuery(ref q, _, _) => Anchor::AfterQuery(
                q.clone(),
                first.sort_field(sort_by)?,
                first.project_id as u32
            ),
            Anchor::Before(..) => Anchor::After(
                first.sort_field(sort_by)?,
                first.project_id as u32
            ),
            Anchor::Start |
            Anchor::After(..) |
            Anchor::StartQuery(..) |
            Anchor::AfterQuery(..) => unreachable!()
        };

        Ok(Some(Seek { anchor: next_anchor, sort_by, dir }))
    }
}

fn get_next_for_after(
    anchor: &Anchor,
    sort_by: SortBy,
    dir: Direction,
    limit_extra: u32,
    projects: &mut Vec<ProjectSummaryRow>
) -> Result<Option<Seek>, AppError>
{
    // make the next link
    if projects.len() == limit_extra as usize {
        // there are more pages in the forward direction

        // remove the "extra" item which proves we are not at the end
        projects.pop();

        // the next page is after the last item
        let last = projects.last().expect("element must exist");

        let next_anchor = match anchor {
            Anchor::StartQuery(ref q) |
            Anchor::AfterQuery(ref q, _, _) => Anchor::AfterQuery(
                q.clone(),
                last.sort_field(sort_by)?,
                last.project_id as u32
            ),
            Anchor::Start |
            Anchor::After(..) => Anchor::After(
                last.sort_field(sort_by)?,
                last.project_id as u32
            ),
            Anchor::Before(..) |
            Anchor::BeforeQuery(..) => unreachable!()
        };

        Ok(Some(Seek { anchor: next_anchor, sort_by, dir }))
    }
    else {
        // there are no pages in the forward direction
        Ok(None)
    }
}

fn get_prev_for_after(
    anchor: &Anchor,
    sort_by: SortBy,
    dir: Direction,
    projects: &Vec<ProjectSummaryRow>
) -> Result<Option<Seek>, AppError>
{
    // make the prev link
    match anchor {
        _ if projects.is_empty() => Ok(None),
        Anchor::Start |
        Anchor::StartQuery(_) => Ok(None),
        Anchor::After(..) |
        Anchor::AfterQuery(..) => {
            // the previous page is before the first item
            let first = projects.first().expect("element must exist");

            let prev_anchor = match anchor {
                Anchor::AfterQuery(ref q, _, _) => Anchor::BeforeQuery(
                    q.clone(),
                    first.sort_field(sort_by)?,
                    first.project_id as u32
                ),
                Anchor::After(..) => Anchor::Before(
                    first.sort_field(sort_by)?,
                    first.project_id as u32
                ),
                Anchor::Start |
                    Anchor::StartQuery(..) |
                Anchor::Before(..) |
                Anchor::BeforeQuery(..) => unreachable!()
            };

            Ok(Some(Seek { anchor: prev_anchor, sort_by, dir }))
        },
        Anchor::Before(..) |
        Anchor::BeforeQuery(..) => unreachable!()
    }
}

fn get_links(
    anchor: &Anchor,
    sort_by: SortBy,
    dir: Direction,
    limit_extra: u32,
    projects: &mut Vec<ProjectSummaryRow>
) -> Result<(Option<Seek>, Option<Seek>), AppError>
{
    match anchor {
        Anchor::Before(..) |
        Anchor::BeforeQuery(..) => {
            let prev = get_prev_for_before(
                anchor,
                sort_by,
                dir,
                limit_extra,
                projects
            )?;

            let next = get_next_for_before(
                anchor,
                sort_by,
                dir,
                projects
            )?;

            Ok((prev, next))
        },
        Anchor::Start |
        Anchor::StartQuery(..) |
        Anchor::After(..) |
        Anchor::AfterQuery(..) => {
            let next = get_next_for_after(
                anchor,
                sort_by,
                dir,
                limit_extra,
                projects
            )?;

            let prev = get_prev_for_after(
                anchor,
                sort_by,
                dir,
                projects
            )?;

            Ok((prev, next))
        }
    }
}

impl ProjectSummaryRow {
    fn sort_field(&self, sort_by: SortBy) -> Result<String, AppError> {
        match sort_by {
            SortBy::ProjectName => Ok(self.name.clone()),
            SortBy::GameTitle => Ok(self.game_title_sort.clone()),
            SortBy::ModificationTime => nanos_to_rfc3339(self.modified_at),
            SortBy::CreationTime => nanos_to_rfc3339(self.created_at),
            SortBy::Relevance => Ok(self.rank.to_string())
        }
    }
}

impl TryFrom<ProjectSummaryRow> for ProjectSummary {
    type Error = AppError;

    fn try_from(r: ProjectSummaryRow) -> Result<Self, Self::Error> {
        Ok(
            ProjectSummary {
                name: r.name,
                description: r.description,
                revision: r.revision,
                created_at: nanos_to_rfc3339(r.created_at)?,
                modified_at: nanos_to_rfc3339(r.modified_at)?,
                tags: vec![],
                game: GameData {
                    title: r.game_title,
                    title_sort_key: r.game_title_sort,
                    publisher: r.game_publisher,
                    year: r.game_year
                }
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

    use crate::{
        model::GameDataPatch,
        pagination::Direction,
        sqlite::{Pool, SqlxDatabaseClient},
        upload::UploadError
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
            created_at: "1970-01-01T00:00:00+00:00".into(),
            modified_at: format!(
                "1970-01-01T00:00:00.0000000{:02}+00:00",
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

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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
    async fn get_projects_pname_before_asc_no_prev_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::Before("d".into(), 4)
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

        assert_eq!(total, 10);

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
    async fn get_projects_pname_after_desc_no_prev_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::Before("g".into(), 7)
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

        assert_eq!(total, 10);

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
    async fn get_projects_pname_after_asc_no_next_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Ascending,
                anchor: Anchor::After("g".into(), 7)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("h"),
                fake_project_summary("i"),
                fake_project_summary("j")
            ]
        );

        assert_eq!(total, 10);

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("h".into(), 8),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(next, None);
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_pname_after_desc_no_next_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ProjectName,
                dir: Direction::Descending,
                anchor: Anchor::After("d".into(), 4)
            },
            Limit::new(3).unwrap()
        ).await.unwrap();

        assert_eq!(
            summaries,
            [
                fake_project_summary("c"),
                fake_project_summary("b"),
                fake_project_summary("a")
            ]
        );

        assert_eq!(total, 10);

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before("c".into(), 3),
                    sort_by: SortBy::ProjectName,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(next, None);
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_start_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

        assert_eq!(prev, None);

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After(
                        "1970-01-01T00:00:00.000000008+00:00".into(),
                        8
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_end_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
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

        assert_eq!(total, 10);

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

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Ascending,
                anchor: Anchor::After(
                    "1970-01-01T00:00:00.000000001+00:00".into(),
                    1
                )
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

        assert_eq!(total, 10);

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before(
                        "1970-01-01T00:00:00.000000002+00:00".into(),
                        2
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After(
                        "1970-01-01T00:00:00.000000004+00:00".into(),
                        4
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_after_desc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Descending,
                anchor: Anchor::After(
                    "1970-01-01T00:00:00.000000008+00:00".into(),
                    8
                )
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

        assert_eq!(total, 10);

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before(
                        "1970-01-01T00:00:00.000000007+00:00".into(),
                        7
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After(
                        "1970-01-01T00:00:00.000000005+00:00".into(),
                        5
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_before_asc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Ascending,
                anchor: Anchor::Before(
                    "1970-01-01T00:00:00.000000005+00:00".into(),
                    5
                )
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

        assert_eq!(total, 10);

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before(
                        "1970-01-01T00:00:00.000000002+00:00".into(),
                        2
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After(
                        "1970-01-01T00:00:00.000000004+00:00".into(),
                        4
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Ascending
                }
            )
        );
    }

    #[sqlx::test(fixtures("users", "ten_projects"))]
    async fn get_projects_mtime_before_desc_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let (prev, next, summaries, total) = core.get_projects_from(
            Seek {
                sort_by: SortBy::ModificationTime,
                dir: Direction::Descending,
                anchor: Anchor::Before(
                    "1970-01-01T00:00:00.000000006+00:00".into(),
                    5
                )
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

        assert_eq!(total, 10);

        assert_eq!(
            prev,
            Some(
                Seek {
                    anchor: Anchor::Before(
                        "1970-01-01T00:00:00.000000008+00:00".into(),
                        8
                    ),
                    sort_by: SortBy::ModificationTime,
                    dir: Direction::Descending
                }
            )
        );

        assert_eq!(
            next,
            Some(
                Seek {
                    anchor: Anchor::After(
                        "1970-01-01T00:00:00.000000006+00:00".into(),
                        6
                    ),
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
            core.get_project(Project(42)).await.unwrap(),
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
            core.get_project_revision(Project(42), 3).await.unwrap(),
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
            core.get_project_revision(Project(42), 1).await.unwrap(),
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

        let user = User(1);
        let name = "newproj";
        let data = ProjectData {
            name: name.into(),
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

        core.create_project(user, &name, &cdata).await.unwrap();
        let proj = core.get_project_id(&name).await.unwrap();
        assert_eq!(core.get_project(proj).await.unwrap(), data);
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn update_project_ok(pool: Pool) {
        let core = make_core(pool, fake_now);

        let name = "test_game";
        let new_data = ProjectData {
            name: name.into(),
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

        let proj = core.get_project_id(&name).await.unwrap();
        let old_data = core.get_project(proj).await.unwrap();
        core.update_project(Owner(1), Project(42), &cdata).await.unwrap();
        // project has new data
        assert_eq!(core.get_project(proj).await.unwrap(), new_data);
        // old data is kept as a revision
        assert_eq!(
            core.get_project_revision(proj, 3).await.unwrap(),
            old_data
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_release(Project(42), Package(1)).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_version_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let version = "1.2.3".parse::<Version>().unwrap();
        assert_eq!(
            core.get_release_version(Project(42), Package(1), &version)
                .await
                .unwrap(),
            "https://example.com/a_package-1.2.3"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_version_not_a_version(pool: Pool) {
        let core = make_core(pool, fake_now);
        let version = "1.0.0".parse::<Version>().unwrap();
        assert_eq!(
            core.get_release_version(Project(42), Package(1), &version)
                .await
                .unwrap_err(),
            AppError::NotAVersion
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn get_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_owners(Project(42)).await.unwrap(),
            Users { users: vec!["bob".into()] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn user_is_owner_true(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(core.user_is_owner(User(1), Project(42)).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn user_is_owner_false(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert!(!core.user_is_owner(User(2), Project(42)).await.unwrap());
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn add_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec!["alice".into()] };
        core.add_owners(&users, Project(42)).await.unwrap();
        assert_eq!(
            core.get_owners(Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into()
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "two_owners"))]
    async fn remove_owners_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec!["bob".into()] };
        core.remove_owners(&users, Project(42)).await.unwrap();
        assert_eq!(
            core.get_owners(Project(42)).await.unwrap(),
            Users { users: vec!["alice".into()] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "one_owner"))]
    async fn remove_owners_fail_if_last(pool: Pool) {
        let core = make_core(pool, fake_now);
        let users = Users { users: vec!["bob".into()] };
        assert_eq!(
            core.remove_owners(&users, Project(1)).await.unwrap_err(),
            AppError::CannotRemoveLastOwner
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn get_players_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_players(Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into()
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn add_player_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        core.add_player(User(3), Project(42)).await.unwrap();
        assert_eq!(
            core.get_players(Project(42)).await.unwrap(),
            Users {
                users: vec![
                    "alice".into(),
                    "bob".into(),
                    "chuck".into()
                ]
            }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "players"))]
    async fn remove_player_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        core.remove_player(User(1), Project(42)).await.unwrap();
        assert_eq!(
            core.get_players(Project(42)).await.unwrap(),
            Users { users: vec!["alice".into()] }
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_ok(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(Project(42), "img.png").await.unwrap(),
            "https://example.com/images/img.png"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_not_a_project(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(Project(1), "img.png").await.unwrap_err(),
            AppError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_not_an_image(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(Project(42), "bogus").await.unwrap_err(),
            AppError::NotFound
        );
    }
}
