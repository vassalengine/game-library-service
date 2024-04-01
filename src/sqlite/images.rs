use sqlx::{
    Acquire, Executor,
    sqlite::Sqlite
};

use crate::{
    core::CoreError,
    model::{Owner, Project},
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

// TODO: tests
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

// TODO: tests
async fn update_image_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64
) -> Result<(), CoreError>
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

// TODO: tests
async fn create_image_revision_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64
) -> Result<(), CoreError>
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

// TODO: tests
pub async fn add_image_url<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    now: i64,
) -> Result<(), CoreError>
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
}
