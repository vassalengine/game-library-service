use async_tempfile::TempFile;
use async_trait::async_trait;
use axum::body::Bytes;
use chrono::{DateTime, Utc};
use futures::Stream;
use futures_util::future::try_join_all;
use mime::Mime;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    future::Future,
    io,
    path::{Path, PathBuf}
};
use tokio::io::AsyncSeekExt;

use crate::{
    core::{Core, CoreError},
    db::{DatabaseClient, FileRow, PackageRow, ProjectRow, ProjectSummaryRow, ReleaseRow},
    model::{FileData, GalleryImage, GameData, Owner, Package, PackageData, PackageDataPost, ProjectData, ProjectDataPatch, ProjectDataPost, Project, Projects, ProjectSummary, Range, RangePatch, Release, ReleaseData, User, Users},
    module::check_version,
    pagination::{Anchor, Direction, Limit, SortBy, Pagination, Seek, SeekLink},
    params::ProjectsParams,
    time::nanos_to_rfc3339,
    upload::{InvalidFilename, LocalUploader, Uploader, UploadError, safe_filename, stream_to_writer},
    version::Version
};

#[derive(Clone)]
pub struct ProdCore<C: DatabaseClient, U: Uploader> {
    pub db: C,
    pub uploader: U,
    pub now: fn() -> DateTime<Utc>,
    pub max_file_size: usize,
    pub max_image_size: usize,
    pub uploads_dir: PathBuf
}

#[async_trait]
impl<C, U> Core for ProdCore<C, U>
where
    C: DatabaseClient + Send + Sync,
    U: Uploader + Send + Sync
{
    fn max_file_size(&self) -> usize {
        self.max_file_size
    }

    fn max_image_size(&self) -> usize {
        self.max_image_size
    }

    async fn get_user_id(
         &self,
        username: &str
    ) -> Result<User, CoreError>
    {
        Ok(self.db.get_user_id(username).await?)
    }

    async fn get_project_id(
         &self,
        proj: &str
    ) -> Result<Project, CoreError>
    {
        self.db.get_project_id(proj).await
    }

    async fn get_owners(
        &self,
        proj: Project
    ) -> Result<Users, CoreError>
    {
        self.db.get_owners(proj).await
    }

    async fn add_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), CoreError>
    {
        self.db.add_owners(owners, proj).await
    }

    async fn remove_owners(
        &self,
        owners: &Users,
        proj: Project
    ) -> Result<(), CoreError>
    {
        self.db.remove_owners(owners, proj).await
    }

    async fn user_is_owner(
        &self,
        user: User,
        proj: Project
    ) -> Result<bool, CoreError>
    {
        self.db.user_is_owner(user, proj).await
    }

    async fn get_projects(
        &self,
        params: ProjectsParams
    ) -> Result<Projects, CoreError>
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
    ) -> Result<ProjectData, CoreError>
    {
        self.get_project_impl(
            proj,
            self.db.get_project_row(proj).await?,
            self.db.get_tags(proj).await?,
            self.db.get_gallery(proj).await?,
            self.db.get_packages(proj).await?,
            |pc, pkg| pc.db.get_releases(pkg),
            |pc, rel| pc.db.get_files(rel)
        ).await
    }

// TODO: length limits on strings
// TODO: packages might need display names?

    async fn create_project(
        &self,
        user: User,
        proj: &str,
        proj_data: &ProjectDataPost
    ) -> Result<(), CoreError>
    {
        let now = self.now_nanos()?;
        self.db.create_project(user, proj, proj_data, now).await
    }

    async fn update_project(
        &self,
        owner: Owner,
        proj: Project,
        proj_data: &ProjectDataPatch
    ) -> Result<(), CoreError>
    {
        let now = self.now_nanos()?;
        self.db.update_project(owner, proj, proj_data, now).await
    }

    async fn get_project_revision(
        &self,
        proj: Project,
        revision: i64
    ) -> Result<ProjectData, CoreError>
    {
        let proj_row = self.db.get_project_row_revision(proj, revision)
            .await?;
        let mtime = proj_row.modified_at;

        let tags = self.db.get_tags_at(proj, mtime).await?;
        let gallery = self.db.get_gallery_at(proj, mtime).await?;
        let package_rows = self.db.get_packages_at(proj, mtime).await?;

        self.get_project_impl(
            proj,
            proj_row,
            tags,
            gallery,
            package_rows,
            |pc, pkg| pc.db.get_releases_at(pkg, mtime),
            |pc, rel| pc.db.get_files_at(rel, mtime)
        ).await
    }

    async fn get_package_id(
         &self,
        proj: Project,
        pkg: &str
    ) -> Result<Package, CoreError>
    {
        self.db.get_package_id(proj, pkg).await
    }

    async fn get_project_package_ids(
         &self,
        proj: &str,
        pkg: &str
    ) -> Result<(Project, Package), CoreError>
    {
        self.db.get_project_package_ids(proj, pkg).await
    }

    async fn create_package(
        &self,
        owner: Owner,
        proj: Project,
        pkg: &str,
        pkg_data: &PackageDataPost
    ) -> Result<(), CoreError>
    {
        let now = self.now_nanos()?;
        self.db.create_package(owner, proj, pkg, pkg_data, now).await
    }

/*
    async fn get_release(
        &self,
        _proj: Project,
        pkg: Package
    ) -> Result<String, CoreError>
    {
        self.db.get_release_url(pkg).await
    }

    async fn get_release_version(
        &self,
        _proj: Project,
        pkg: Package,
        version: &Version
    ) -> Result<String, CoreError>
    {
        self.db.get_release_version_url(pkg, version).await
    }
*/

    async fn get_release_id(
        &self,
        proj: Project,
        pkg: Package,
        release: &str
    ) -> Result<Release, CoreError>
    {
        self.db.get_release_id(proj, pkg, release).await
    }

    async fn create_release(
        &self,
        owner: Owner,
        proj: Project,
        pkg: Package,
        version: &Version,
    ) -> Result<(), CoreError>
    {
        let now = self.now_nanos()?;
        self.db.create_release(owner, proj, pkg, version, now).await
    }

    async fn add_file(
        &self,
        owner: Owner,
        proj: Project,
        release: Release,
        requires: Option<&str>,
        filename: &str,
        content_length: Option<u64>,
        stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>
    ) -> Result<(), CoreError>
    {
        let now = self.now_nanos()?;

        // ensure the filename is valid
        let filename = safe_filename(filename)
            .or(Err(CoreError::InvalidFilename))?;

        // write the stream to a file
        let mut file = TempFile::new_in(&*self.uploads_dir)
            .await
            .map_err(io::Error::other)?;

        let stream = Box::into_pin(stream);

        let (sha256, size) = stream_to_writer(stream, &mut file)
            .await?;

        // check that the content length, if given, matches what was read
        if content_length.is_some_and(|cl| cl != size) {
            // we know the stream is shorter because were it longer, it
            // would already have failed from hitting the limit
            return Err(
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "stream shorter than Content-Length"
                )
            )?;
        }

        // check that vmod, vext files have semver-compliant versions
        let ext = Path::new(filename).extension().unwrap_or_default();
        if ext == "vmod" || ext == "vext" {
            check_version(file.file_path()).await?;
        }

// TODO: should uploaded files be named for hashes?

        // add hash prefix to file upload path
        let bucket_path = format!(
            "{0}/{1}/{filename}",
            &sha256[0..1],
            &sha256[1..2]
        );

        file.rewind().await?;

// TODO: do we need to set content-type on upload?
        let url = self.uploader.upload(
            &bucket_path,
            &mut file
        )
        .await?;

        // update record
        self.db.add_file_url(
            owner,
            proj,
            release,
            filename,
            size as i64,
            &sha256,
            requires,
            &url,
            now
        ).await?;

        Ok(())
    }

    async fn get_players(
        &self,
        proj: Project
    ) -> Result<Users, CoreError>
    {
        self.db.get_players(proj).await
    }

    async fn add_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), CoreError>
    {
        self.db.add_player(player, proj).await
    }

    async fn remove_player(
        &self,
        player: User,
        proj: Project
    ) -> Result<(), CoreError>
    {
        self.db.remove_player(player, proj).await
    }

    async fn get_image(
        &self,
        proj: Project,
        img_name: &str
    ) -> Result<String, CoreError>
    {
        self.db.get_image_url(proj, img_name).await
    }

    async fn get_image_revision(
        &self,
        proj: Project,
        revision: i64,
        img_name: &str
    ) -> Result<String, CoreError>
    {
        let proj_row = self.db.get_project_row_revision(proj, revision).await?;
        let mtime = proj_row.modified_at;
        self.db.get_image_url_at(proj, img_name, mtime).await
    }

// TODO: tests
    async fn add_image(
        &self,
        owner: Owner,
        proj: Project,
        filename: &str,
        content_type: &Mime,
        content_length: Option<u64>,
        stream: Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>
    ) -> Result<(), CoreError>
    {
        let now = self.now_nanos()?;

        // MIME type should be an images
        if !image_mime_type_ok(content_type) {
            return Err(CoreError::BadMimeType);
        }

        // ensure the filename is valid
        let filename = safe_filename(filename)
            .or(Err(CoreError::InvalidFilename))?;

        // write the stream to a file
        let mut file = TempFile::new_in(&*self.uploads_dir)
            .await
            .map_err(io::Error::other)?;

        let mut stream = Box::into_pin(stream);

// TODO: check actual MIME type
// TODO: check dimensions? resize?
/*
        let mut stream = std::pin::pin!(stream.peekable());
        let chunk = stream.as_mut().peek()
            .await
            .ok_or(CoreError::InternalError)?
            .as_ref()
            .or(Err(CoreError::InternalError))?;

        let magic = infer::get(chunk.as_ref())
            .ok_or(CoreError::BadMimeType)?
            .mime_type()
            .parse::<Mime>()
            .or(Err(CoreError::BadMimeType))?;

        if magic != *content_type {
            return Err(CoreError::BadMimeType);
        }
*/

        let (sha256, size) = stream_to_writer(stream, &mut file)
            .await?;

        // check that the content length, if given, matches what was read
        if content_length.is_some_and(|cl| cl != size) {
            // we know the stream is shorter because were it longer, it
            // would already have failed from hitting the limit
            return Err(
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "stream shorter than Content-Length"
                )
            )?;
        }

// TODO: should uploaded files be named for hashes?

        // add hash prefix to file upload path
        let bucket_path = format!(
            "{0}/{1}/{filename}",
            &sha256[0..1],
            &sha256[1..2]
        );

        file.rewind().await?;

// TODO: do we need to set content-type on upload?
        let url = self.uploader.upload(
            &bucket_path,
            &mut file
        )
        .await?;

        // update record
        self.db.add_image_url(owner, proj, filename, &url, now).await?;

        Ok(())
    }
}

fn image_mime_type_ok(mime: &Mime) -> bool {
    mime == &mime::IMAGE_PNG ||
    mime == &mime::IMAGE_GIF ||
    mime == &mime::IMAGE_JPEG ||
    mime == &mime::IMAGE_SVG ||
    (
        mime.type_() == mime::IMAGE && (
            mime.subtype() == "avif" ||
            mime.subtype() == "webp"
        )
    )
}

fn range_if_some(min: Option<i64>, max: Option<i64>) -> Option<Range> {
    if min.is_some() || max.is_some() {
        Some(Range { min, max })
    }
    else {
        None
    }
}

impl<C, U> ProdCore<C, U>
where
    C: DatabaseClient + Send + Sync,
    U: Uploader + Send + Sync
{
    fn now_nanos(&self) -> Result<i64, CoreError> {
        (self.now)()
            .timestamp_nanos_opt()
            .ok_or(CoreError::InternalError)
    }

    async fn make_file_data(
        &self,
        r: FileRow
    ) -> Result<FileData, CoreError>
    {
        let authors = self.db.get_authors(r.id)
            .await?
            .users;

        Ok(
            FileData {
                filename: r.filename,
                url: r.url,
                size: r.size,
                sha256: r.sha256,
                published_at: nanos_to_rfc3339(r.published_at)?,
                published_by: r.published_by,
                requires: r.requires,
                authors
            }
        )
    }

    async fn make_release_data<'s, FF, FR>(
        &'s self,
        rr: ReleaseRow,
        get_files_rows: &FF
    ) -> Result<ReleaseData, CoreError>
    where
        FF: Fn(&'s Self, Release) -> FR,
        FR: Future<Output = Result<Vec<FileRow>, CoreError>>
    {
        let files = try_join_all(
            get_files_rows(self, Release(rr.release_id))
                .await?
                .into_iter()
                .map(|fr| self.make_file_data(fr))
        ).await?;

        Ok(
            ReleaseData {
                version: rr.version,
                files
            }
        )
    }

    async fn make_package_data<'s, RF, RR, FF, FR>(
        &'s self,
        pr: PackageRow,
        get_release_rows: &RF,
        get_files_rows: &FF
    ) -> Result<PackageData, CoreError>
    where
        RF: Fn(&'s Self, Package) -> RR,
        RR: Future<Output = Result<Vec<ReleaseRow>, CoreError>>,
        FF: Fn(&'s Self, Release) -> FR,
        FR: Future<Output = Result<Vec<FileRow>, CoreError>>
    {
        let releases = try_join_all(
            get_release_rows(self, Package(pr.package_id))
                .await?
                .into_iter()
                .map(|rr| self.make_release_data(
                    rr,
                    &get_files_rows
                ))
        ).await?;

        Ok(
            PackageData {
                name: pr.name,
                description: "".into(),
                releases
            }
        )
    }

    async fn get_project_impl<'s, RF, RR, FF, FR>(
        &'s self,
        proj: Project,
        proj_row: ProjectRow,
        tags: Vec<String>,
        gallery: Vec<GalleryImage>,
        package_rows: Vec<PackageRow>,
        get_release_rows: RF,
        get_file_rows: FF,
    ) -> Result<ProjectData, CoreError>
    where
        RF: Fn(&'s Self, Package) -> RR,
        RR: Future<Output = Result<Vec<ReleaseRow>, CoreError>>,
        FF: Fn(&'s Self, Release) -> FR,
        FR: Future<Output = Result<Vec<FileRow>, CoreError>>
    {
        let owners = self.get_owners(proj)
            .await?
            .users;

        let packages = try_join_all(
            package_rows
                .into_iter()
                .map(|pr| self.make_package_data(
                    pr,
                    &get_release_rows,
                    &get_file_rows
                ))
        ).await?;

        Ok(
            ProjectData {
                name: proj_row.name,
                description: proj_row.description,
                revision: proj_row.revision,
                created_at: nanos_to_rfc3339(proj_row.created_at)?,
                modified_at: nanos_to_rfc3339(proj_row.modified_at)?,
                tags,
                game: GameData {
                    title: proj_row.game_title,
                    title_sort_key: proj_row.game_title_sort,
                    publisher: proj_row.game_publisher,
                    year: proj_row.game_year,
                    players: range_if_some(
                        proj_row.game_players_min,
                        proj_row.game_players_max
                    ),
                    length: range_if_some(
                        proj_row.game_length_min,
                        proj_row.game_length_max
                    )
                },
                readme: proj_row.readme,
                image: proj_row.image,
                owners,
                packages,
                gallery
            }
        )
    }

    async fn get_projects_window(
        &self,
        anchor: &Anchor,
        sort_by: SortBy,
        dir: Direction,
        limit_extra: u32
    ) -> Result<Vec<ProjectSummaryRow>, CoreError>
    {
        match anchor {
            Anchor::Start =>
                self.db.get_projects_end_window(
                    sort_by,
                    dir,
                    limit_extra
                ).await,
            Anchor::After(field, id) =>
                self.db.get_projects_mid_window(
                    sort_by,
                    dir,
                    field,
                    *id,
                    limit_extra
                ).await,
            Anchor::Before(field, id) =>
                self.db.get_projects_mid_window(
                    sort_by,
                    dir.rev(),
                    field,
                    *id,
                    limit_extra
                ).await,
            Anchor::StartQuery(query) =>
                self.db.get_projects_query_end_window(
                    query,
                    sort_by,
                    dir,
                    limit_extra
                ).await,
            Anchor::AfterQuery(query, field, id) =>
                self.db.get_projects_query_mid_window(
                    query,
                    sort_by,
                    dir,
                    field,
                    *id,
                    limit_extra
                ).await,
            Anchor::BeforeQuery(query, field, id) =>
                self.db.get_projects_query_mid_window(
                    query,
                    sort_by,
                    dir.rev(),
                    field,
                    *id,
                    limit_extra
                ).await
        }
    }

    async fn get_projects_from(
        &self,
        seek: Seek,
        limit: Limit
    ) -> Result<(Option<Seek>, Option<Seek>, Vec<ProjectSummary>, i64), CoreError>
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
            Anchor::AfterQuery(ref q, ..) |
            Anchor::BeforeQuery(ref q, ..) =>
                self.db.get_projects_query_count(q).await,
            _ => self.db.get_projects_count().await
        }?;

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

fn check_new_project_name(projname: &str) -> Result<(), CoreError> {
    // Require that project name matches ^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$
    static PAT: Lazy<Regex> = Lazy::new(||
        Regex::new("^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")
            .expect("bad regex")
    );

    if !PAT.is_match(projname) {
        Err(CoreError::InvalidProjectName)
    }
    else {
        Ok(())
    }
}

fn get_prev_for_before(
    anchor: &Anchor,
    sort_by: SortBy,
    dir: Direction,
    limit_extra: u32,
    projects: &mut Vec<ProjectSummaryRow>
) -> Result<Option<Seek>, CoreError>
{
    // make the prev link
    if projects.len() == limit_extra as usize {
        // there are more pages in the forward direction

        // remove the "extra" item which proves we are not at the end
        projects.pop();

        // the prev page is after the last item
        let last = projects.last().expect("element must exist");

        let prev_anchor = match anchor {
            Anchor::BeforeQuery(ref q, ..) => Anchor::BeforeQuery(
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
    projects: &[ProjectSummaryRow]
) -> Result<Option<Seek>, CoreError>
{
    // make the next link
    if projects.is_empty() {
        Ok(None)
    }
    else {
        // the next page is before the first item
        let first = projects.first().expect("element must exist");

        let next_anchor = match anchor {
            Anchor::BeforeQuery(ref q, ..) => Anchor::AfterQuery(
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
) -> Result<Option<Seek>, CoreError>
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
            Anchor::AfterQuery(ref q, ..) => Anchor::AfterQuery(
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
    projects: &[ProjectSummaryRow]
) -> Result<Option<Seek>, CoreError>
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
                Anchor::AfterQuery(ref q, ..) => Anchor::BeforeQuery(
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
) -> Result<(Option<Seek>, Option<Seek>), CoreError>
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
    fn sort_field(&self, sort_by: SortBy) -> Result<String, CoreError> {
        Ok(
            match sort_by {
                SortBy::ProjectName => self.name.clone(),
                SortBy::GameTitle => self.game_title_sort.clone(),
                SortBy::ModificationTime => nanos_to_rfc3339(self.modified_at)?,
                SortBy::CreationTime => nanos_to_rfc3339(self.created_at)?,
                SortBy::Relevance => self.rank.to_string()
            }
        )
    }
}

impl TryFrom<ProjectSummaryRow> for ProjectSummary {
    type Error = CoreError;

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
                    year: r.game_year,
                    players: None,
                    length: None
                }
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use tokio::io::AsyncRead;

    use crate::{
        model::GameDataPatch,
        pagination::Direction,
        sqlite::{Pool, SqlxDatabaseClient},
        upload::UploadError
    };

    const NOW: &str = "2023-11-12T15:50:06.419538067+00:00";

    static NOW_DT: Lazy<DateTime<Utc>> = Lazy::new(||
        DateTime::parse_from_rfc3339(NOW)
            .unwrap()
            .with_timezone(&Utc)
    );

    fn fake_now() -> DateTime<Utc> {
        *NOW_DT
    }

    struct FakeUploader {}

    impl Uploader for FakeUploader {
        async fn upload<R>(
            &self,
            _filename: &str,
            _reader: R
        ) -> Result<String, UploadError>
        where
            R: AsyncRead + Unpin + Send
        {
            unreachable!();
        }
    }

    fn make_core(
        pool: Pool,
        now: fn() -> DateTime<Utc>
    ) -> ProdCore<SqlxDatabaseClient<sqlx::sqlite::Sqlite>, FakeUploader>
    {
        ProdCore {
            db: SqlxDatabaseClient(pool),
            uploader: FakeUploader {},
            now,
            max_file_size: 256,
            max_image_size: 256,
            uploads_dir: "uploads".into()
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
                year: "".into(),
                players: None,
                length: None
            }
        }
    }

    #[test]
    fn check_new_project_name_ok() {
        check_new_project_name("acceptable_name").unwrap();
    }

    #[test]
    fn check_new_project_name_non_ascii() {
        assert_eq!(
            check_new_project_name("💩").unwrap_err(),
            CoreError::InvalidProjectName
        );
    }

    #[test]
    fn check_new_project_name_leading_non_alphanumeric() {
        assert_eq!(
            check_new_project_name("-abc").unwrap_err(),
            CoreError::InvalidProjectName
        );
    }

    #[test]
    fn check_new_project_name_too_short() {
        assert_eq!(
            check_new_project_name("").unwrap_err(),
            CoreError::InvalidProjectName
        );
    }

    #[test]
    fn check_new_project_name_too_long() {
        assert_eq!(
            check_new_project_name(&"x".repeat(100)).unwrap_err(),
            CoreError::InvalidProjectName
        );
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
                    year: "1979".into(),
                    players: None,
                    length: None
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
                                files: vec![
                                    FileData {
                                        filename: "a_package-1.2.4".into(),
                                        url: "https://example.com/a_package-1.2.4".into(),
                                        size: 5678,
                                        sha256: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
                                        published_at: "2023-12-10T15:56:29.180282477+00:00".into(),
                                        published_by: "alice".into(),
                                        requires: Some(">= 3.7.12".into()),
                                        authors: vec!["alice".into(), "bob".into()]
                                    },
                                ],
                            },
                            ReleaseData {
                                version: "1.2.3".into(),
                                files: vec![
                                    FileData {
                                        filename: "a_package-1.2.3".into(),
                                        url: "https://example.com/a_package-1.2.3".into(),
                                        size: 1234,
                                        sha256: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
                                        published_at: "2023-12-09T15:56:29.180282477+00:00".into(),
                                        published_by: "bob".into(),
                                        requires: Some(">= 3.2.17".into()),
                                        authors: vec!["alice".into()]
                                    }
                                ]
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
                                files: vec![
                                    FileData {
                                        filename: "c_package-0.1.0".into(),
                                        url: "https://example.com/c_package-0.1.0".into(),
                                        size: 123456,
                                        sha256: "a8f515e9e2de99919d1a987733296aaa951a4ba2aa0f7014c510bdbd60dc0efd".into(),
                                        published_at: "2023-12-15T15:56:29.180282477+00:00".into(),
                                        published_by: "chuck".into(),
                                        requires: None,
                                        authors: vec![]
                                    }
                                ]
                            }
                        ]
                    }
                ],
                gallery: vec![]
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
                    year: "1979".into(),
                    players: None,
                    length: None
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
                                files: vec![
                                    FileData {
                                        filename: "a_package-1.2.4".into(),
                                        url: "https://example.com/a_package-1.2.4".into(),
                                        size: 5678,
                                        sha256: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
                                        published_at: "2023-12-10T15:56:29.180282477+00:00".into(),
                                        published_by: "alice".into(),
                                        requires: Some(">= 3.7.12".into()),
                                        authors: vec!["alice".into(), "bob".into()]
                                    },
                                ]
                            },
                            ReleaseData {
                                version: "1.2.3".into(),
                                files: vec![
                                    FileData {
                                        filename: "a_package-1.2.3".into(),
                                        url: "https://example.com/a_package-1.2.3".into(),
                                        size: 1234,
                                        sha256: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
                                        published_at: "2023-12-09T15:56:29.180282477+00:00".into(),
                                        published_by: "bob".into(),
                                        requires: Some(">= 3.2.17".into()),
                                        authors: vec!["alice".into()]
                                    }
                                ]
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
                ],
                gallery: vec![]
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
                    year: "1978".into(),
                    players: None,
                    length: None
                },
                readme: "".into(),
                image: None,
                owners: vec!["alice".into(), "bob".into()],
                packages: vec![
                    PackageData {
                        name: "b_package".into(),
                        description: "".into(),
                        releases: vec![],
                    },
                    PackageData {
                        name: "c_package".into(),
                        description: "".into(),
                        releases: vec![],
                    }
                ],
                gallery: vec![]
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
                year: "1999".into(),
                players: None,
                length: None
            },
            readme: "".into(),
            image: None,
            owners: vec!["bob".into()],
            packages: vec![],
            gallery: vec![]
        };

        let cdata = ProjectDataPost {
            description: data.description.clone(),
            tags: vec![],
            game: GameData {
                title: data.game.title.clone(),
                title_sort_key: data.game.title_sort_key.clone(),
                publisher: data.game.publisher.clone(),
                year: data.game.year.clone(),
                players: None,
                length: None
            },
            readme: "".into(),
            image: None
        };

        core.create_project(user, name, &cdata).await.unwrap();
        let proj = core.get_project_id(name).await.unwrap();
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
                year: "1999".into(),
                players: None,
                length: None,
            },
            readme: "".into(),
            image: None,
            owners: vec!["bob".into()],
            packages: vec![],
            gallery: vec![]
        };

        let cdata = ProjectDataPatch {
            description: Some(new_data.description.clone()),
            tags: Some(vec![]),
            game: GameDataPatch {
                title: Some(new_data.game.title.clone()),
                title_sort_key: Some(new_data.game.title_sort_key.clone()),
                publisher: Some(new_data.game.publisher.clone()),
                year: Some(new_data.game.year.clone()),
                players: None,
                length: None
            },
            readme: Some("".into()),
            image: None
        };

        let proj = core.get_project_id(name).await.unwrap();
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

/*
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
            CoreError::NotAVersion
        );
    }
*/

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
            CoreError::CannotRemoveLastOwner
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
            CoreError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_not_an_image(pool: Pool) {
        let core = make_core(pool, fake_now);
        assert_eq!(
            core.get_image(Project(42), "bogus").await.unwrap_err(),
            CoreError::NotFound
        );
    }
}
