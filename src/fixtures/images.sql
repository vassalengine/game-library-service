INSERT INTO images (
  project_id,
  filename,
  url,
  published_at,
  published_by
)
VALUES
  (
    42,
    "img.png",
    "https://example.com/images/img.png",
    1694804206419538067,
    1
  );

INSERT INTO image_revisions (
  project_id,
  filename,
  url,
  published_at,
  published_by
)
VALUES
  (
    42,
    "img.png",
    "https://example.com/images/img.png",
    1694804206419538067,
    1
  );
