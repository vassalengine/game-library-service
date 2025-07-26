use sqlx::{
    Acquire, Executor, QueryBuilder,
    sqlite::Sqlite
};

use crate::{
    db::{DatabaseError, PackageRow, map_unique},
    input::{PackageDataPatch, PackageDataPost, slug_for},
    model::{Owner, Package,  Project},
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
    slug,
    sort_key,
    created_at
FROM packages
WHERE project_id = ?
ORDER BY sort_key ASC
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
    // NB: sqlx can't figure out that all the columns here are NOT NULL,
    // so we have to annotate them in the result
    Ok(
        sqlx::query_as!(
            PackageRow,
            r#"
SELECT
    packages_history.package_id AS "package_id!",
    packages_revisions.name AS "name!",
    packages_revisions.slug AS "slug!",
    packages_revisions.sort_key AS "sort_key!",
    packages_history.created_at AS "created_at!"
FROM packages_history
JOIN packages_revisions
ON packages_history.package_id = packages_revisions.package_id
WHERE packages_history.project_id = ?
    AND packages_history.created_at <= ?
    AND (? < packages_history.deleted_at OR packages_history.deleted_at IS NULL)
    AND packages_revisions.modified_at <= ?
GROUP BY packages_history.package_id
HAVING MAX(packages_revisions.modified_at)
ORDER BY sort_key ASC
            "#,
            proj.0,
            date,
            date,
            date
        )
       .fetch_all(ex)
       .await?
    )
}

pub async fn get_package_id<'e, E>(
    ex: E,
    proj: Project,
    pkgslug: &str
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
    AND slug = ?
            ",
            proj.0,
            pkgslug
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

async fn create_package_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    pkg: Package,
    name: &str,
    slug: &str,
    sort_key: i64,
    now: i64
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO packages (
    package_id,
    project_id,
    name,
    slug,
    sort_key,
    created_at,
    created_by
)
VALUES (?, ?, ?, ?, ?, ?, ?)
        ",
        pkg.0,
        proj.0,
        name,
        slug,
        sort_key,
        now,
        owner.0
    )
    .execute(ex)
    .await
    .map_err(map_unique)?;

    Ok(())
}

async fn create_package_history_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    now: i64
) -> Result<Package, DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
INSERT INTO packages_history (
    project_id,
    created_at,
    created_by
)
VALUES (?, ?, ?)
RETURNING package_id
        ",
        proj.0,
        now,
        owner.0
    )
    .fetch_one(ex)
    .await
    .map(Package)
    .map_err(map_unique)
}

async fn create_package_revision_row<'e, E>(
    ex: E,
    owner: Owner,
    pkg: Package,
    name: &str,
    slug: &str,
    sort_key: i64,
    now: i64
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO packages_revisions (
    package_id,
    name,
    slug,
    sort_key,
    modified_at,
    modified_by
)
VALUES (?, ?, ?, ?, ?, ?)
        ",
        pkg.0,
        name,
        slug,
        sort_key,
        now,
        owner.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn create_package<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    slug: &str,
    pkg_data: &PackageDataPost,
    now: i64
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    let sort_key = pkg_data.sort_key;

    let pkg = create_package_history_row(&mut *tx, owner, proj, now).await?;

    create_package_revision_row(
        &mut *tx,
        owner,
        pkg,
        &pkg_data.name,
        slug,
        sort_key,
        now
    ).await?;

    create_package_row(
        &mut *tx,
        owner,
        proj,
        pkg,
        &pkg_data.name,
        slug,
        sort_key,
        now
    ).await?;

    update_project_non_project_data(&mut tx, owner, proj, now).await?;

    tx.commit().await?;

    Ok(())
}

async fn update_package_row<'e, E>(
    ex: E,
    pkg: Package,
    pd: &PackageDataPatch,
    now: i64
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    let mut qb: QueryBuilder<E::Database> = QueryBuilder::new(
        "UPDATE packages SET "
    );

    let mut qbs = qb.separated(", ");

    if let Some(name) = &pd.name {
        qbs
            .push("name = ")
            .push_bind_unseparated(name)
            .push("slug = ")
            .push_bind_unseparated(slug_for(name));
    }

    if let Some(sort_key) = &pd.sort_key {
        qbs.push("sort_key = ").push_bind_unseparated(sort_key);
    }

    if let Some(description) = &pd.description {
        qbs.push("description = ").push_bind_unseparated(description);
    }

    qb
        .push(" WHERE package_id = ")
        .push_bind(pkg.0)
        .build()
        .execute(ex)
        .await?;

    Ok(())
}

async fn update_package_revision_row<'e, E>(
    ex: E,
    owner: Owner,
    pkg: Package,
    now: i64
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO packages_revisions (
    package_id,
    name,
    slug,
    sort_key,
    modified_at,
    modified_by
)
SELECT
    package_id,
    name,
    slug,
    sort_key,
    ? AS modified_at,
    ? AS modified_by
FROM packages
WHERE package_id = ?
        ",
        now,
        owner.0,
        pkg.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn update_package<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    pkg: Package,
    pkg_data: &PackageDataPatch,
    now: i64
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    update_package_row(&mut *tx, pkg, pkg_data, now).await?;
    update_package_revision_row(&mut *tx, owner, pkg, now).await?;
    update_project_non_project_data(&mut tx, owner, proj, now).await?;

    tx.commit().await?;

    Ok(())
}

async fn check_package_row_exists<'e, E>(
    ex: E,
    pkg: Package,
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT 1 FROM packages
WHERE package_id = ?
        ",
        pkg.0
    )
    .fetch_optional(ex)
    .await?
    .and(Some(()))
    .ok_or(DatabaseError::NotFound)
}

async fn delete_package_row<'e, E>(
    ex: E,
    pkg: Package,
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
DELETE FROM packages
WHERE package_id = ?
        ",
        pkg.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn retire_package_history_row<'e, E>(
    ex: E,
    owner: Owner,
    pkg: Package,
    now: i64
) -> Result<(), DatabaseError>
where
 E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
UPDATE packages_history
SET
    deleted_by = ?,
    deleted_at = ?
WHERE package_id = ?
        ",
        owner.0,
        now,
        pkg.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete_package<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    pkg: Package,
    now: i64
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    check_package_row_exists(&mut *tx, pkg).await?;

    delete_package_row(&mut *tx, pkg).await?;
    retire_package_history_row(&mut *tx, owner, pkg, now).await?;
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
                    slug: "a_package".into(),
                    sort_key: 0,
                    created_at: 1702137389180282477
                },
                PackageRow {
                    package_id: 2,
                    name: "b_package".into(),
                    slug: "b_package".into(),
                    sort_key: 1,
                    created_at: 1667750189180282477
                },
                PackageRow {
                    package_id: 3,
                    name: "c_package".into(),
                    slug: "c_package".into(),
                    sort_key: 2,
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
                    slug: "b_package".into(),
                    sort_key: 1,
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
            get_packages_at(&pool, proj, 1699804206419538067).await.unwrap(),
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
                name: "newpkg".into(),
                sort_key: 4,
                description: "".into()
            },
            1699804206419538067
        ).await.unwrap();

        let pr = PackageRow {
            package_id: 5,
            name: "newpkg".into(),
            slug: "newpkg".into(),
            sort_key: 4,
            created_at: 1699804206419538067
        };

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            [ pr.clone() ]
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1699804206419538067).await.unwrap(),
            [ pr.clone() ]
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
                        name: "newpkg".into(),
                        sort_key: 4,
                        description: "".into()
                    },
                    1699804206419538067
                ).await.unwrap_err(),
                DatabaseError::SqlxError(_)
            )
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn create_package_already_exists(pool: Pool) {
        assert_eq!(
            create_package(
                &pool,
                Owner(1),
                Project(42),
                "a_package",
                &PackageDataPost {
                    name: "a_package".into(),
                    sort_key: 4,
                    description: "".into()
                },
                1699804206419538067
            ).await.unwrap_err(),
            DatabaseError::AlreadyExists
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn update_package_ok(pool: Pool) {
        let proj = Project(42);

        let prs_before = [
            PackageRow {
                package_id: 1,
                name: "a_package".into(),
                slug: "a_package".into(),
                sort_key: 0,
                created_at: 1702137389180282477
            },
            PackageRow {
                package_id: 2,
                name: "b_package".into(),
                slug: "b_package".into(),
                sort_key: 1,
                created_at: 1667750189180282477
            },
            PackageRow {
                package_id: 3,
                name: "c_package".into(),
                slug: "c_package".into(),
                sort_key: 2,
                created_at: 1699286189180282477
            }
        ];

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            prs_before
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1702137389180282477).await.unwrap(),
            prs_before
        );

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap().revision,
            3
        );

        update_package(
            &pool,
            Owner(1),
            Project(42),
            Package(1),
            &PackageDataPatch {
                sort_key: Some(4),
                ..Default::default()
            },
            1702137389180282478
        ).await.unwrap();

        let prs_after = [
            PackageRow {
                package_id: 2,
                name: "b_package".into(),
                slug: "b_package".into(),
                sort_key: 1,
                created_at: 1667750189180282477
            },
            PackageRow {
                package_id: 3,
                name: "c_package".into(),
                slug: "c_package".into(),
                sort_key: 2,
                created_at: 1699286189180282477
            },
            PackageRow {
                package_id: 1,
                name: "a_package".into(),
                slug: "a_package".into(),
                sort_key: 4,
                created_at: 1702137389180282477
            }
        ];

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            prs_after
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1702137389180282477).await.unwrap(),
            prs_before
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1702137389180282478).await.unwrap(),
            prs_after
        );

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap().revision,
            4
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn delete_package_ok(pool: Pool) {
        let proj = Project(42);

        let prs_before = [
            PackageRow {
                package_id: 1,
                name: "a_package".into(),
                slug: "a_package".into(),
                sort_key: 0,
                created_at: 1702137389180282477
            },
            PackageRow {
                package_id: 2,
                name: "b_package".into(),
                slug: "b_package".into(),
                sort_key: 1,
                created_at: 1667750189180282477
            },
            PackageRow {
                package_id: 3,
                name: "c_package".into(),
                slug: "c_package".into(),
                sort_key: 2,
                created_at: 1699286189180282477
            }
        ];

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            prs_before
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1702137389180282477).await.unwrap(),
            prs_before
        );

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap().revision,
            3
        );

        delete_package(
            &pool,
            Owner(1),
            Project(42),
            Package(2),
            1702137389180282478
        ).await.unwrap();

        let prs_after = [
            PackageRow {
                package_id: 1,
                name: "a_package".into(),
                slug: "a_package".into(),
                sort_key: 0,
                created_at: 1702137389180282477
            },
            PackageRow {
                package_id: 3,
                name: "c_package".into(),
                slug: "c_package".into(),
                sort_key: 2,
                created_at: 1699286189180282477
            }
        ];

        assert_eq!(
            get_packages(&pool, proj).await.unwrap(),
            prs_after
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1702137389180282477).await.unwrap(),
            prs_before
        );

        assert_eq!(
            get_packages_at(&pool, proj, 1702137389180282478).await.unwrap(),
            prs_after
        );

        assert_eq!(
            get_project_row(&pool, proj).await.unwrap().revision,
            4
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn delete_package_not_project(pool: Pool) {
        assert_eq!(
            delete_package(
                &pool,
                Owner(1),
                Project(0),
                Package(2),
                1702137389180282478
            ).await.unwrap_err(),
            DatabaseError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn delete_package_not_package(pool: Pool) {
        assert_eq!(
            delete_package(
                &pool,
                Owner(1),
                Project(42),
                Package(5),
                1702137389180282478
            ).await.unwrap_err(),
            DatabaseError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "packages"))]
    async fn delete_package_not_empty(pool: Pool) {
        assert!(
            matches!(
                delete_package(
                    &pool,
                    Owner(1),
                    Project(42),
                    Package(1),
                    1702137389180282478
                ).await.unwrap_err(),
                DatabaseError::SqlxError(_)
            )
        );
    }
}
