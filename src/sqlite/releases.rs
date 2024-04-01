use serde::Deserialize;
use sqlx::{
    Executor,
    sqlite::Sqlite
};
use std::cmp::Ordering;

use crate::{
    core::CoreError,
    db::ReleaseRow,
    model::Package,
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

#[derive(Debug, Deserialize)]
struct ReducedReleaseRow {
    url: String,
    version_major: i64,
    version_minor: i64,
    version_patch: i64,
    version_pre: String,
    version_build: String,
}

impl From<&ReducedReleaseRow> for Version {
    fn from(r: &ReducedReleaseRow) -> Self {
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
) -> Result<Vec<ReleaseRow>, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReleaseRow,
        "
SELECT
    releases.release_id,
    releases.version,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build,
    releases.url,
    releases.filename,
    releases.size,
    releases.checksum,
    releases.published_at,
    users.username AS published_by
FROM releases
JOIN users
ON releases.published_by = users.user_id
WHERE package_id = ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC,
    version_pre ASC,
    version_build ASC
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
) -> Result<Vec<ReleaseRow>, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut releases = sqlx::query_as!(
        ReleaseRow,
        "
SELECT
    releases.release_id,
    releases.version,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build,
    releases.url,
    releases.filename,
    releases.size,
    releases.checksum,
    releases.published_at,
    users.username AS published_by
FROM releases
JOIN users
ON releases.published_by = users.user_id
WHERE package_id = ?
    AND published_at <= ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC,
    version_pre ASC,
    version_build ASC
        ",
        pkg.0,
        date
    )
    .fetch_all(ex)
    .await?;

    releases.sort_by(release_row_desc_cmp);
    Ok(releases)
}

pub async fn get_release_version_url<'e, E>(
    ex: E,
    pkg: Package,
    version: &Version
) -> Result<String, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    let pre = version.pre.as_deref().unwrap_or("");
    let build = version.build.as_deref().unwrap_or("");

    sqlx::query_scalar!(
        "
SELECT url
FROM releases
WHERE package_id = ?
    AND version_major = ?
    AND version_minor = ?
    AND version_patch = ?
    AND version_pre = ?
    AND version_build = ?
LIMIT 1
        ",
        pkg.0,
        version.major,
        version.minor,
        version.patch,
        pre,
        build
    )
    .fetch_optional(ex)
    .await?
    .ok_or(CoreError::NotAVersion)
}

pub async fn get_release_url<'e, E>(
    ex: E,
    pkg: Package
) -> Result<String, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_as!(
        ReducedReleaseRow,
        "
SELECT
    url,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build
FROM releases
WHERE package_id = ?
ORDER BY
    version_major DESC,
    version_minor DESC,
    version_patch DESC,
    version_pre ASC,
    version_build ASC
        ",
        pkg.0
    )
    .fetch_all(ex)
    .await?
    .into_iter()
    .min_by(release_row_desc_cmp)
    .map(|r| r.url)
    .ok_or(CoreError::NotAPackage)
}

// TODO: update project mtime when release is added

#[cfg(test)]
mod test {
    use super::*;

    type Pool = sqlx::Pool<Sqlite>;

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_ok(pool: Pool) {
        assert_eq!(
            get_releases(&pool, Package(1)).await.unwrap(),
            vec![
                ReleaseRow {
                    release_id: 2,
                    version: "1.2.4".into(),
                    version_major: 1,
                    version_minor: 2,
                    version_patch: 4,
                    version_pre: "".into(),
                    version_build: "".into(),
                    url: "https://example.com/a_package-1.2.4".into(),
                    filename: "a_package-1.2.4".into(),
                    size: 5678,
                    checksum: "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2".into(),
                    published_at: 1702223789180282477,
                    published_by: "alice".into()
                },
                ReleaseRow {
                    release_id: 1,
                    version: "1.2.3".into(),
                    version_major: 1,
                    version_minor: 2,
                    version_patch: 3,
                    version_pre: "".into(),
                    version_build: "".into(),
                    url: "https://example.com/a_package-1.2.3".into(),
                    filename: "a_package-1.2.3".into(),
                    size: 1234,
                    checksum: "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a".into(),
                    published_at: 1702137389180282477,
                    published_by: "bob".into()
                }
            ]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_releases_not_a_package(pool: Pool) {
        // This should not happen; the Package passed in should be good.
        // However, it's not an error if it does.
        assert_eq!(
            get_releases(&pool, Package(0)).await.unwrap(),
            vec![]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_url_ok(pool: Pool) {
        assert_eq!(
            get_release_url(&pool, Package(1)).await.unwrap(),
            "https://example.com/a_package-1.2.4"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_release_url_not_a_package(pool: Pool) {
        assert_eq!(
            get_release_url(&pool, Package(0)).await.unwrap_err(),
            CoreError::NotAPackage
        );
    }
}
