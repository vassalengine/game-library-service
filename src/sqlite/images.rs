use sqlx::{
    Acquire, Executor,
    sqlite::Sqlite
};
use std::collections::HashMap;

use crate::{
    db::{DatabaseError, map_unique},
    model::{GalleryImage, GalleryItem, Owner, Project},
    sqlite::project::update_project_non_project_data
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

async fn update_image_row<'e, E>(
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
INSERT INTO images (
    project_id,
    filename,
    url,
    content_type,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?, ?)
ON CONFLICT(project_id, filename)
DO UPDATE
SET url = excluded.url,
    content_type = excluded.content_type,
    published_at = excluded.published_at,
    published_by = excluded.published_by
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

    // update row in images
    update_image_row(
        &mut *tx,
        owner,
        proj,
        img_name,
        url,
        content_type,
        now
    ).await?;

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

#[derive(Clone, Debug, Eq, PartialEq)]
struct GalleryRow {
    gallery_id: i64,
    next_id: Option<i64>,
    filename: String,
    description: String
}

impl From<GalleryRow> for GalleryImage {
    fn from(r: GalleryRow) -> GalleryImage {
        GalleryImage {
            id: r.gallery_id,
            filename: r.filename,
            description: r.description
        }
    }
}

fn sort_as_ll(imgs: &mut [GalleryRow]) -> Result<(), DatabaseError> {
    // NB: It is a precondition that the list head is first in the slice

    // empty slices are already sorted
    if imgs.len() == 0 {
        return Ok(());
    }

    // singletons must have no successor
    if imgs.len() == 1 {
        if imgs[0].next_id == None {
            return Ok(())
        }
        else {
            return Err(DatabaseError::InvalidLinkedList);
        }
    }

    // make a map from ids to indices
    let mut idx: HashMap<i64, usize> = HashMap::from_iter(
        imgs.iter()
            .enumerate()
            .map(|(i, img)| (img.gallery_id, i))
    );

    // chase the next pointers
    for i in 1..(imgs.len() - 1) {
        if let Some(next_id) = imgs[i-1].next_id {
            if let Some(j) = idx.get(&next_id) {
                // swap the item which should be next with
                // the item occupying the next index
                let swap_id = imgs[i].gallery_id;
                imgs.swap(i, *j);
                // update the index map
                idx.insert(swap_id, *j);
            }
            else {
                // imgs[i-1].next_id is a bad index
                return Err(DatabaseError::InvalidLinkedList);
            }
        }
        else {
            // imgs[i-1].next_id should never be None
            return Err(DatabaseError::InvalidLinkedList);
        }
    }

    // check that the tail is the last element
    if imgs[imgs.len() - 1].next_id != None {
        return Err(DatabaseError::InvalidLinkedList);
    }

    Ok(())
}

pub async fn get_gallery<'e, E>(
    ex: E,
    proj: Project
) -> Result<Vec<GalleryImage>, DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let mut imgs = sqlx::query_as!(
        GalleryRow,
        "
SELECT
    gallery_id,
    next_id,
    filename,
    description
FROM galleries
WHERE project_id = ?
ORDER BY prev_id NULLS FIRST
        ",
        proj.0
    )
    .fetch_all(ex)
    .await?;

    sort_as_ll(&mut imgs)?;

    Ok(
        imgs.into_iter()
            .map(|r| r.into())
            .collect::<Vec<_>>()
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
    let mut imgs = sqlx::query_as!(
        GalleryRow,
        "
SELECT
    gallery_id,
    next_id,
    filename,
    description
FROM galleries_history
WHERE project_id = ?
    AND published_at <= ?
    AND (removed_at > ? OR removed_at IS NULL)
ORDER BY prev_id NULLS FIRST
        ",
        proj.0,
        date,
        date
    )
    .fetch_all(ex)
    .await?;

    sort_as_ll(&mut imgs)?;

    Ok(
        imgs.into_iter()
            .map(|r| r.into())
            .collect::<Vec<_>>()
    )
}

async fn create_galleries_history_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    img_name: &str,
    description: &str,
    now: i64
) -> Result<(GalleryItem, Option<GalleryItem>), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    sqlx::query!(
        "
INSERT INTO galleries_history (
    prev_id,
    project_id,
    filename,
    description,
    published_at,
    published_by
)
SELECT
    gallery_id,
    ?,
    ?,
    ?,
    ?,
    ?
FROM galleries_history
WHERE project_id = ?
    AND next_id IS NULL 
RETURNING gallery_id, prev_id
        ",
        proj.0,
        img_name,
        description,
        now,
        owner.0,
        proj.0
    )
    .fetch_one(ex)
    .await
    .map(|r| (GalleryItem(r.gallery_id), r.prev_id.map(GalleryItem)))
    .map_err(map_unique)
}

async fn update_galleries_row<'e, E>(
    ex: E,
    owner: Owner,
    proj: Project,
    item: GalleryItem,
    prev: Option<GalleryItem>,
    next: Option<GalleryItem>,
    img_name: &str,
    description: &str,
    now: i64
) -> Result<(), DatabaseError>
where
    E: Executor<'e, Database = Sqlite>
{
    let prev_id = prev.map(|i| i.0);
    let next_id = next.map(|i| i.0);

    sqlx::query!(
        "
INSERT INTO galleries (
    gallery_id,
    prev_id,
    next_id,
    project_id,
    filename,
    description,
    published_at,
    published_by
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(gallery_id)
DO UPDATE
SET prev_id = excluded.prev_id,
    next_id = excluded.next_id,
    description = excluded.description,
    published_at = excluded.published_at,
    published_by = excluded.published_by
        ",
        item.0,
        prev_id,
        next_id,
        proj.0,
        img_name,
        description,
        now,
        owner.0
    )
    .execute(ex)
    .await?;

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
    fn sort_as_ll_empty() {
        let mut v = vec![];
        sort_as_ll(&mut v).unwrap();
        assert_eq!(&v, &[]);
    }

    #[test]
    fn sort_ll_1() {
        let exp = [
            GalleryRow {
                gallery_id: 1,
                next_id: None,
                filename: "".into(),
                description: "".into()
            }
        ];

        let mut act = exp.clone();
        sort_as_ll(&mut act).unwrap();
        assert_eq!(&act, &exp);
    }

    #[test]
    fn sort_as_ll_1_bad_next() {
        let mut act = [
            GalleryRow {
                gallery_id: 1,
                next_id: Some(3),
                filename: "".into(),
                description: "".into()
            }
        ];
        sort_as_ll(&mut act).unwrap_err();
    }

    #[test]
    fn sort_as_ll_12() {
        let exp = [
            GalleryRow {
                gallery_id: 1,
                next_id: Some(2),
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 2,
                next_id: None,
                filename: "".into(),
                description: "".into()
            }

        ];

        let mut act = exp.clone();
        sort_as_ll(&mut act).unwrap();
        assert_eq!(&act, &exp);
    }

    #[test]
    fn sort_as_ll_123() {
        let exp = [
            GalleryRow {
                gallery_id: 1,
                next_id: Some(2),
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 2,
                next_id: Some(3),
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 3,
                next_id: None,
                filename: "".into(),
                description: "".into()
            }
        ];

        let mut act = exp.clone();
        sort_as_ll(&mut act).unwrap();
        assert_eq!(&act, &exp);
    }

    #[test]
    fn sort_as_ll_132() {
        let exp = [
            GalleryRow {
                gallery_id: 1,
                next_id: Some(3),
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 3,
                next_id: Some(2),
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 2,
                next_id: None,
                filename: "".into(),
                description: "".into()
            },
        ];

        let mut act = [
            GalleryRow {
                gallery_id: 1,
                next_id: Some(3),
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 2,
                next_id: None,
                filename: "".into(),
                description: "".into()
            },
            GalleryRow {
                gallery_id: 3,
                next_id: Some(2),
                filename: "".into(),
                description: "".into()
            }
        ];

        sort_as_ll(&mut act).unwrap();
        assert_eq!(&act, &exp);
    }

    #[test]
    fn sort_as_ll_10() {
        let n = 10;

        let exp = (1..=n).map(|i| GalleryRow {
                gallery_id: i,
                next_id: if i < n { Some(i+1) } else { None },
                filename: "".into(),
                description: "".into()
            })
            .collect::<Vec<_>>();

        let mut act = exp.clone();
        act[1..].reverse();

        sort_as_ll(&mut act).unwrap();
        assert_eq!(&act, &exp);
    }

    #[test]
    fn sort_as_ll_lots() {
        use rand::{
            RngCore, SeedableRng,
            rngs::StdRng,
            seq::SliceRandom
        };

        let n = 1000;

        let exp = (1..=n).map(|i| GalleryRow {
                gallery_id: i,
                next_id: if i < n { Some(i+1) } else { None },
                filename: "".into(),
                description: "".into()
            })
            .collect::<Vec<_>>();

        let mut act = exp.clone();

        let seed = rand::rng().next_u64();
        let mut rng = StdRng::seed_from_u64(seed);

        // shuffle, keeping head in place
        act[1..].shuffle(&mut rng);

        sort_as_ll(&mut act).unwrap();
        assert_eq!(&act, &exp, "{act:?} != {exp:?}, with seed {seed}");
    }
}
