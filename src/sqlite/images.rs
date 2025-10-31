use num_bigint::BigUint;
use sqlx::{
    Acquire, Executor, Transaction,
    sqlite::Sqlite
};
use std::collections::HashMap;

use crate::{
    db::{DatabaseError, map_unique},
    input::{GalleryOp, GalleryPatch},
    model::{GalleryImage, GalleryItem, Owner, Project},
    sqlite::{
        require_one_modified,
        project::update_project_non_project_data
    }
};

pub async fn get_image_url<'e, E>(
    ex: E,
    proj: Project,
    img_name: &str
) -> Result<Option<String>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
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
    )
}

pub async fn get_image_url_at<'e, E>(
    ex: E,
    proj: Project,
    img_name: &str,
    date: i64
) -> Result<Option<String>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    Ok(
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
    )
}

async fn create_image_revision_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    url: &str,
    content_type: &str,
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
    content_type,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?, ?)
        ",
        proj.0,
        img_name,
        url,
        content_type,
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
    content_type: &str,
    now: i64,
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    // insert row in images_revisions
    create_image_revision_row(
        &mut *tx,
        owner,
        proj,
        img_name,
        url,
        content_type,
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
SELECT
    gallery_id AS id,
    filename,
    description
FROM galleries
WHERE project_id = ?
ORDER BY sort_key
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
SELECT
    gallery_id AS id,
    filename,
    description
FROM galleries_history
WHERE project_id = ?
    AND published_at <= ?
    AND (removed_at > ? OR removed_at IS NULL)
ORDER BY sort_key
            ",
            proj.0,
            date,
            date
        )
        .fetch_all(ex)
        .await?
    )
}

async fn update_gallery_item(
    tx: &mut Transaction<'_, Sqlite>,
    owner: Owner,
    proj: Project,
    gallery_id: i64,
    description: &str,
    now: i64
) -> Result<(), DatabaseError>
{
    todo!();
}

async fn delete_gallery_item(
    tx: &mut Transaction<'_, Sqlite>,
    owner: Owner,
    proj: Project,
    gallery_id: i64,
    now: i64
) -> Result<(), DatabaseError>
{
    todo!();
}

async fn move_gallery_item(
    tx: &mut Transaction<'_, Sqlite>,
    owner: Owner,
    proj: Project,
    gallery_id: i64,
    next_id: Option<i64>,
    now: i64
) -> Result<(), DatabaseError>
{
    todo!();
}

fn midpoint(a: &[u8], b: &[u8]) -> Vec<u8> {
    // ensure that a, b have the same length
    let len = std::cmp::max(a.len(), b.len());

    let mut av = Vec::with_capacity(len);
    av.extend(a);
    av.resize(len, 0);

    let bv = if b.len() < len {
        let mut bv = Vec::with_capacity(len);
        bv.extend(b);
        bv.resize(len, 0);
        Some(bv)
    }
    else {
        None
    };

    let a = &av[..];
    let b = if let Some(ref bv) = bv { &bv[..] } else { b };

    let ai = BigUint::from_bytes_be(a);
    let bi = BigUint::from_bytes_be(b);

    // extend if a and b differ by one, otherwise take the average
    if &ai + 1u32 == bi {
        av.push(0x80);
        av
    }
    else {
        let mi: BigUint = (ai + bi) >> 1;
        mi.to_bytes_be()
    }
}

pub async fn update_gallery<'a, A>(
    conn: A,
    owner: Owner,
    proj: Project,
    gallery_patch: &GalleryPatch,
    now: i64
) -> Result<(), DatabaseError>
where
    A: Acquire<'a, Database = Sqlite>
{
    let mut tx = conn.begin().await?;

    for op in &gallery_patch.ops {
        match op {
            GalleryOp::Update { id, description } => update_gallery_item(
                &mut tx, owner, proj, *id, description, now
            ).await,
            GalleryOp::Delete { id } => delete_gallery_item(
                &mut tx, owner, proj, *id, now
            ).await,
            GalleryOp::Move { id, next } => move_gallery_item(
                &mut tx, owner, proj, *id, *next, now
            ).await
        }?;
    }

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
            Some("https://example.com/images/img.png".into())
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_not_a_project(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(1), "img.png").await.unwrap(),
            None
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_not_an_image(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "bogus").await.unwrap(),
            None
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
            Some("https://example.com/images/img.png".into())
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_at_not_a_project(pool: Pool) {
        assert_eq!(
            get_image_url_at(&pool, Project(1), "img.png", 0).await.unwrap(),
            None
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn get_image_url_at_not_an_image(pool: Pool) {
        assert_eq!(
            get_image_url_at(&pool, Project(42), "bogus", 0).await.unwrap(),
            None
        );
    }

    #[sqlx::test(fixtures("users", "projects", "images"))]
    async fn add_image_url_ok(pool: Pool) {
        assert_eq!(
            get_image_url(&pool, Project(42), "image.png").await.unwrap(),
            None
        );

        add_image_url(
            &pool,
            Owner(1),
            Project(42),
            "image.png",
            "https://example.com/image.png",
            "image/png",
            1703980420641538067
        ).await.unwrap();

        assert_eq!(
            get_image_url(&pool, Project(42), "image.png").await.unwrap(),
            Some("https://example.com/image.png".into())
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
                    "image/png",
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
                    "image/png",
                    0
                ).await.unwrap_err(),
                DatabaseError::SqlxError(_)
            )
        );
    }

    #[test]
    fn midpoint_00_ff() {
        assert_eq!(midpoint(&[0x00], &[0xFF]), &[0x7F]);
    }

    #[test]
    fn midpoint_00_fe() {
        assert_eq!(midpoint(&[0x00], &[0xFE]), &[0x7F]);
    }

    #[test]
    fn midpoint_00_01() {
        assert_eq!(midpoint(&[0x00], &[0x01]), &[0x00, 0x80]);
    }

    #[sqlx::test(fixtures("users", "projects", "images", "galleries"))]
    async fn update_gallery_delete(pool: Pool) {
        assert_eq!(
            get_gallery(&pool, Project(42)).await.unwrap(),
            [
                GalleryImage {
                    id: 1, 
                    filename: "img.png".into(),
                    description: "".into()
                }
            ]
        );

        update_gallery(
            &pool,
            Owner(1),
            Project(42),
            &GalleryPatch {
                ops: vec![
                    GalleryOp::Delete { id: 1 }
                ]
            }, 
            1703980420641538067
        ).await.unwrap();

        assert_eq!(
            get_gallery(&pool, Project(42)).await.unwrap(),
            []
        );
    }
}
