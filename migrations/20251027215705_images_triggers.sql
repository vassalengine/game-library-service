COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TRIGGER IF NOT EXISTS image_revisions_ai_start
AFTER INSERT ON image_revisions
BEGIN
  INSERT OR REPLACE INTO images (
    project_id,
    filename,
    url,
    content_type,
    published_at,
    published_by
  )
  VALUES (
    NEW.project_id,
    NEW.filename,
    NEW.url,
    NEW.content_type,
    NEW.published_at,
    NEW.published_by
  );
END;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
