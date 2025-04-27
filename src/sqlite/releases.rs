use sqlx::{
    Acquire, Executor,
    sqlite::Sqlite
};
use std::cmp::Ordering;

use crate::{
    db::{DatabaseError, FileRow, ReleaseRow, map_unique},
    model::{Owner, Package, Project, Release},
    sqlite::project::update_project_non_project_data,
    version::Version
};

impl From<&ReleaseRow> for Version {
    fn from(r: &ReleaseRow) -> Self {
        Version {
            major: r.version_major,
            minor: r.version_minor,
            patch: r.version_patch,
            pre: Some(&r.version_pre).filter(|v| !v.is_empty()).cloned(),
            build: Some(&r.version_build).filter(|v| !v.is_empty()).cloned()
        }
    }
}

fn release_row_desc_cmp<R>(a: &R, b: &R) -> Ordering
where
    Version: for<'r> From<&'r R>
{
    let av: Version = a.into();
    let bv: Version = b.into();
    bv.cmp(&av)
}

pub async fn get_releases<'e, E>(
    ex: E,
    pkg: Package
) -> Result<Vec<ReleaseRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReleaseRow,
        "
SELECT
    releases.release_id,
    releases.version,
    releases.version_major,
    releases.version_minor,
    releases.version_patch,
    releases.version_pre,
    releases.version_build,
    releases.published_at,
    users.username AS published_by
FROM releases
JOIN users
ON releases.published_by = users.user_id
WHERE releases.package_id = ?
ORDER BY
    releases.version_major DESC,
    releases.version_minor DESC,
    releases.version_patch DESC,
    releases.version_pre ASC,
    releases.version_build ASC
        ",
        pkg.0
    )
    .fetch_all(ex)
    .await?;

    releases.sort_by(release_row_desc_cmp);
    Ok(releases)
}

pub async fn get_releases_at<'e, E>(
    ex: E,
    pkg: Package,
    date: i64
) -> Result<Vec<ReleaseRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReleaseRow,
        "
SELECT
    releases.release_id,
    releases.version,
    releases.version_major,
    releases.version_minor,
    releases.version_patch,
    releases.version_pre,
    releases.version_build,
    releases.published_at,
    users.username AS published_by
FROM releases
JOIN users
ON releases.published_by = users.user_id
WHERE releases.package_id = ?
    AND releases.published_at <= ?
ORDER BY
    releases.version_major DESC,
    releases.version_minor DESC,
    releases.version_patch DESC,
    releases.version_pre ASC,
    releases.version_build ASC
        ",
        pkg.0,
        date
    )
    .fetch_all(ex)
    .await?;

    releases.sort_by(release_row_desc_cmp);
    Ok(releases)
}

pub async fn get_release_id<'e, E>(
    ex: E,
    proj: Project,
    pkg: Package,
    release: &str
) -> Result<Option<Release>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT release_id
FROM releases
WHERE package_id = ?
    AND version = ?
            ",
            pkg.0,
            release
        )
        .fetch_optional(ex)
        .await?
        .map(Release)
    )
}

pub async fn get_project_package_release_ids<'e, E>(
    ex: E,
    projname: &str,
    pkgname: &str,
    release: &str
) -> Result<Option<(Project, Package, Release)>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT packages.project_id,
    packages.package_id,
    releases.release_id
FROM projects
JOIN packages
ON projects.project_id = packages.project_id
JOIN releases
ON packages.package_id = releases.package_id
WHERE projects.name = ?
    AND packages.name = ?
    AND releases.version = ?
            ",
            projname,
            pkgname,
            release
        )
        .fetch_optional(ex)
        .await?
        .map(|r| (
            Project(r.project_id),
            Package(r.package_id),
            Release(r.release_id)
        ))
    )
}

pub async fn create_release<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    pkg: Package,
    version: &Version,
    now: i64
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // insert release row
    create_release_row(&mut *tx, owner, pkg, version, now).await?;

    // update project to reflect the change
    update_project_non_project_data(&mut tx, owner, proj, now).await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_files<'e, E>(
    ex: E,
    release: Release
) -> Result<Vec<FileRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_as!(
            FileRow,
            "
SELECT
    files.file_id AS id,
    files.url,
    files.filename,
    files.size,
    files.sha256,
    files.requires,
    files.published_at,
    users.username AS published_by
FROM files
JOIN users
ON files.published_by = users.user_id
WHERE files.release_id = ?
ORDER BY
    files.filename ASC
            ",
            release.0
        )
        .fetch_all(ex)
        .await?
    )
}

pub async fn get_files_at<'e, E>(
    ex: E,
    release: Release,
    date: i64
) -> Result<Vec<FileRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_as!(
            FileRow,
            "
SELECT
    files.file_id AS id,
    files.url,
    files.filename,
    files.size,
    files.sha256,
    files.requires,
    files.published_at,
    users.username AS published_by
FROM files
JOIN users
ON files.published_by = users.user_id
WHERE files.release_id = ?
    AND files.published_at <= ?
ORDER BY
    files.filename ASC
            ",
            release.0,
            date
        )
        .fetch_all(ex)
        .await?
    )
}

async fn create_release_row<'e, E>(
    ex: E,
    owner: Owner,
    pkg: Package,
    version: &Version,
    now: i64
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let vstr = String::from(version);
    let pre = version.pre.as_deref().unwrap_or("");
    let build = version.build.as_deref().unwrap_or("");

    sqlx::query!(
        "
INSERT INTO releases (
    package_id,
    version,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        pkg.0,
        vstr,
        version.major,
        version.minor,
        version.patch,
        pre,
        build,
        now,
        owner.0
    )
    .execute(ex)
    .await
    .map_err(map_unique)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn create_file_row<'e, E>(
    ex: E,
    owner: Owner,
    release: Release,
    filename: &str,
    size: i64,
    sha256: &str,
    requires: Option<&str>,
    url: &str,
    now: i64
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO files (
    release_id,
    url,
    filename,
    size,
    sha256,
    requires,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        release.0,
        url,
        filename,
        size,
        sha256,
        requires,
        now,
        owner.0
    )
    .execute(ex)
    .await
    .map_err(map_unique)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn add_file_url<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    release: Release,
    filename: &str,
    size: i64,
    sha256: &str,
    requires: Option<&str>,
    url: &str,
    now: i64
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // insert file row
    create_file_row(
        &mut *tx,
        owner,
        release,
        filename,
        size,
        sha256,
        requires,
        url,
        now
    ).await?;

    // update project to reflect the change
    update_project_non_project_data(&mut tx, owner, proj, now).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use once_cell::sync::Lazy;

    type Pool = sqlx::Pool<Sqlite>;

    static RR_1_2_3: Lazy<ReleaseRow> = Lazy::new(||
        ReleaseRow {
            release_id: 1,
            version: "1.2.3".into(),
            version_major: 1,
            version_minor: 2,
            version_patch: 3,
            version_pre: "".into(),
            version_build: "".into(),
            published_at: 1702137389180282477,
            published_by: "bob".into()
        }
    );

/*
        FileRow {
            id: 1,
            url: "https://example.com/a_package-1.2.3".into(),
            filename: "a_package-1.2.3".into(),
            size: 1234,
            sha256: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
            requires: ">= 3.2.17".into(),
            published_at: 1702137389180282477,
            published_by: "bob".into()
        }
*/

    static RR_1_2_4: Lazy<ReleaseRow> = Lazy::new(||
        ReleaseRow {
            release_id: 2,
            version: "1.2.4".into(),
            version_major: 1,
            version_minor: 2,
            version_patch: 4,
            version_pre: "".into(),
            version_build: "".into(),
            published_at: 1702223789180282477,
            published_by: "alice".into()
        }
    );

/*
        FileRow {
            id: 2,
            version: "1.2.4".into(),
            version_major: 1,
            version_minor: 2,
            version_patch: 4,
            version_pre: "".into(),
            version_build: "".into(),
            url: "https://example.com/a_package-1.2.4".into(),
            filename: "a_package-1.2.4".into(),
            size: 5678,
            sha256: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
            requires: ">= 3.7.12".into(),
            published_at: 1702223789180282477,
            published_by: "alice".into()
        }
*/

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_ok(pool: Pool) {
        assert_eq!(
            get_releases(&pool, Package(1)).await.unwrap(),
            [ RR_1_2_4.clone(), RR_1_2_3.clone() ]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_not_a_package(pool: Pool) {
        // This should not happen; the Package passed in should be good.
        // However, it's not an error if it does.
        assert_eq!(
            get_releases(&pool, Package(0)).await.unwrap(),
            []
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_at_all(pool: Pool) {
        assert_eq!(
            get_releases_at(&pool, Package(1), 1705223789180282477)
                .await
                .unwrap(),
            [ RR_1_2_4.clone(), RR_1_2_3.clone() ]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_at_some(pool: Pool) {
        assert_eq!(
            get_releases_at(&pool, Package(1), 1702137399180282477)
                .await
                .unwrap(),
            [ RR_1_2_3.clone() ]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_at_none(pool: Pool) {
        assert_eq!(
            get_releases_at(&pool, Package(1), 0).await.unwrap(),
            []
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_at_not_a_package(pool: Pool) {
        // This should not happen; the Package passed in should be good.
        // However, it's not an error if it does.
        assert_eq!(
            get_releases_at(&pool, Package(0), 0).await.unwrap(),
            []
        );
    }

// TODO: create_release tests
// TODO: add_file_url tests
}
