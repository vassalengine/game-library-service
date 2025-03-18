use sqlx::{
    Acquire, Executor,
    sqlite::Sqlite
};

use crate::{
    core::CoreError,
    db::DatabaseError,
    model::{GalleryImage, Owner, Project},
    sqlite::project::update_project_non_project_data
};

pub async fn get_image_url<'e, E>(
    ex: E,
    proj: Project,
    img_name: &str
) -> Result<String, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT url
FROM images
WHERE project_id = ?
    AND filename = ?
LIMIT 1
        ",
        proj.0,
        img_name
    )
    .fetch_optional(ex)
    .await?
    .ok_or(CoreError::NotFound)
}

pub async fn get_image_url_at<'e, E>(
    ex: E,
    proj: Project,
    img_name: &str,
    date: i64
) -> Result<String, CoreError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query_scalar!(
        "
SELECT url
FROM image_revisions
WHERE project_id = ?
    AND filename = ?
    AND published_at <= ?
ORDER BY published_at DESC
LIMIT 1
        ",
        proj.0,
        img_name,
        date
    )
    .fetch_optional(ex)
    .await?
    .ok_or(CoreError::NotFound)
}

async fn update_image_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO images (
    project_id,
    filename,
    url,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?)
ON CONFLICT(project_id, filename)
DO UPDATE
SET url = excluded.url,
    published_at = excluded.published_at,
    published_by = excluded.published_by
        ",
        proj.0,
        img_name,
        url,
        now,
        owner.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

async fn create_image_revision_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO image_revisions (
    project_id,
    filename,
    url,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?)
        ",
        proj.0,
        img_name,
        url,
        now,
        owner.0
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn add_image_url<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64,
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // update row in images
    update_image_row(
        &mut *tx,
        owner,
        proj,
        img_name,
        url,
        now
    ).await?;

    // insert row in images_revisions
    create_image_revision_row(
        &mut *tx,
        owner,
        proj,
        img_name,
        url,
        now
    ).await?;

    // update project to reflect the change
    update_project_non_project_data(&mut tx, owner, proj, now).await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_gallery<'e, E>(
    ex: E,
    proj: Project
) -> Result<Vec<GalleryImage>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
       sqlx::query_as!(
            GalleryImage,
            "
SELECT filename, description
FROM galleries
WHERE project_id = ?
    AND removed_at IS NULL
ORDER BY position
            ",
            proj.0
        )
        .fetch_all(ex)
        .await?
    )
}

pub async fn get_gallery_at<'e, E>(
    ex: E,
    proj: Project,
    date: i64
) -> Result<Vec<GalleryImage>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
   Ok(
       sqlx::query_as!(
            GalleryImage,
            "
SELECT filename, description
FROM galleries
WHERE project_id = ?
    AND published_at <= ?
    AND (removed_at > ? OR removed_at IS NULL)
ORDER BY position
            ",
            proj.0,
            date,
            date
        )
        .fetch_all(ex)
        .await?
    )
}

#[cfg(test)]
mod test {
    use super::*;

    type Pool = sqlx::Pool<Sqlite>;

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_ok(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "img.png").await.unwrap(),
            "https://example.com/images/img.png"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_not_a_project(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(1), "img.png").await.unwrap_err(),
            CoreError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_not_an_image(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "bogus").await.unwrap_err(),
            CoreError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_at_ok(pool: Pool) {
        assert_eq!(
            get_image_url_at(
                &pool,
                Project(42),
                "img.png",
                1712012874000000000
            ).await.unwrap(),
            "https://example.com/images/img.png"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_at_not_a_project(pool: Pool) {
        assert_eq!(
            get_image_url_at(&pool, Project(1), "img.png", 0).await.unwrap_err(),
            CoreError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_at_not_an_image(pool: Pool) {
        assert_eq!(
            get_image_url_at(&pool, Project(42), "bogus", 0).await.unwrap_err(),
            CoreError::NotFound
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn add_image_url_ok(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "image.png").await.unwrap_err(),
            CoreError::NotFound
        );

        add_image_url(
            &pool,
            Owner(1),
            Project(42),
            "image.png",
            "https://example.com/image.png",
            1703980420641538067
        ).await.unwrap();

        assert_eq!(
            get_image_url(&pool, Project(42), "image.png").await.unwrap(),
            "https://example.com/image.png"
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn add_image_url_not_a_user(pool: Pool) {
        // This should not happen; the Owner passed in should be good.
        assert!(
            matches!(
                add_image_url(
                    &pool,
                    Owner(0),
                    Project(42),
                    "image.png",
                    "https://example.com/image.png",
                    0
                ).await.unwrap_err(),
                DatabaseError::SqlxError(_)
            )
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn add_image_url_not_a_project(pool: Pool) {
        // This should not happen; the Project passed in should be good.
        assert!(
            matches!(
                add_image_url(
                    &pool,
                    Owner(1),
                    Project(0),
                    "image.png",
                    "https://example.com/image.png",
                    0
                ).await.unwrap_err(),
                DatabaseError::SqlxError(_)
            )
        );
    }
}
