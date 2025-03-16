use sqlx::{
    Acquire, Executor,
    sqlite::Sqlite
};

use crate::{
    core::CoreError,
    db::{DatabaseError, PackageRow},
    model::{Owner, Package, PackageDataPost, Project},
    sqlite::project::update_project_non_project_data
};

pub async fn get_packages<'e, E>(
    ex: E,
    proj: Project
) -> Result<Vec<PackageRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_as!(
            PackageRow,
            "
SELECT
    package_id,
    name,
    created_at
FROM packages
WHERE project_id = ?
ORDER BY name COLLATE NOCASE ASC
            ",
            proj.0
        )
       .fetch_all(ex)
       .await?
    )
}

pub async fn get_packages_at<'e, E>(
    ex: E,
    proj: Project,
    date: i64
) -> Result<Vec<PackageRow>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_as!(
            PackageRow,
            "
SELECT
    package_id,
    name,
    created_at
FROM packages
WHERE project_id = ?
    AND created_at <= ?
ORDER BY name COLLATE NOCASE ASC
            ",
            proj.0,
            date
        )
       .fetch_all(ex)
       .await?
    )
}

pub async fn get_package_id<'e, E>(
    ex: E,
    proj: Project,
    pkgname: &str
) -> Result<Option<Package>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query_scalar!(
            "
SELECT package_id
FROM packages
WHERE project_id = ?
    AND name = ?
            ",
            proj.0,
            pkgname
        )
        .fetch_optional(ex)
        .await?
        .map(Package)
    )
}

pub async fn get_project_package_ids<'e, E>(
    ex: E,
    projname: &str,
    pkgname: &str
) -> Result<Option<(Project, Package)>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
        sqlx::query!(
            "
SELECT packages.project_id,
    packages.package_id
FROM projects
JOIN packages
ON projects.project_id = packages.project_id
WHERE projects.name = ?
    AND packages.name = ?
            ",
            projname,
            pkgname
        )
        .fetch_optional(ex)
        .await?
        .map(|r| (Project(r.project_id), Package(r.package_id)))
    )
}

pub async fn create_package<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    pkg: &str,
    pkg_data: &PackageDataPost,
    now: i64
) -> Result<(), CoreError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    sqlx::query!(
        "
INSERT INTO packages (
    project_id,
    name,
    created_at,
    created_by
)
VALUES (?, ?, ?, ?)
            ",
            proj.0,
            pkg,
            now,
            owner.0
    )
    .execute(&mut *tx)
    .await?;

    // update project to reflect the change
    update_project_non_project_data(&mut tx, owner, proj, now).await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::sqlite::project::get_project_row;

    type Pool = sqlx::Pool<Sqlite>;

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_ok(pool: Pool) {
        assert_eq!(
            get_packages(&pool, Project(42)).await.unwrap(),
            [
                PackageRow {
                    package_id: 1,
                    name: "a_package".into(),
                    created_at: 1702137389180282477
                },
                PackageRow {
                    package_id: 2,
                    name: "b_package".into(),
                    created_at: 1667750189180282477
                },
                PackageRow {
                    package_id: 3,
                    name: "c_package".into(),
                    created_at: 1699286189180282477
                }
            ]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_not_a_project(pool: Pool) {
        // This should not happen; the Project passed in should be good.
        // However, it's not an error if it does.
        assert_eq!(
            get_packages(&pool, Project(0)).await.unwrap(),
            []
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_at_none(pool: Pool) {
        let date = 0;
        assert_eq!(
            get_packages_at(&pool, Project(42), date).await.unwrap(),
            []
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_at_some(pool: Pool) {
        let date = 1672531200000000000;
        assert_eq!(
            get_packages_at(&pool, Project(42), date).await.unwrap(),
            [
                PackageRow {
                    package_id: 2,
                    name: "b_package".into(),
                    created_at: 1667750189180282477
                }
            ]
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn get_packages_at_not_a_project(pool: Pool) {
        // This should not happen; the Project passed in should be good.
        // However, it's not an error if it does.
        let date = 16409952000000000;
        assert_eq!(
            get_packages_at(&pool, Project(0), date).await.unwrap(),
            []
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn create_package_ok(pool: Pool) {
        let proj = Project(6);

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            []
        );

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap().revision,
            1
        );

        create_package(
            &pool,
            Owner(1),
            proj,
            "newpkg",
            &PackageDataPost {
                description: "".into()
            },
            1699804206419538067
        ).await.unwrap();

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            [
                PackageRow {
                    package_id: 4,
                    name: "newpkg".into(),
                    created_at: 1699804206419538067
                }
            ]
        );

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap().revision,
            2
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn create_package_not_a_project(pool: Pool) {
        assert!(
            matches!(
                create_package(
                    &pool,
                    Owner(1),
                    Project(0),
                    "newpkg",
                    &PackageDataPost {
                        description: "".into()
                    },
                    1699804206419538067
                ).await.unwrap_err(),
                CoreError::DatabaseError(_)
            )
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn create_package_already_exists(pool: Pool) {
        assert!(
            matches!(
                create_package(
                    &pool,
                    Owner(1),
                    Project(42),
                    "a_package",
                    &PackageDataPost {
                        description: "".into()
                    },
                    1699804206419538067
                ).await.unwrap_err(),
                CoreError::DatabaseError(_)
            )
        );
    }
}
