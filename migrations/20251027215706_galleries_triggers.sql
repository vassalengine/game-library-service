COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TRIGGER IF NOT EXISTS galleries_history_ai_start
AFTER INSERT ON galleries_history
BEGIN
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
  VALUES (
    NEW.gallery_id,
    NEW.prev_id,
    NEW.next_id,
    NEW.project_id,
    NEW.filename,
    NEW.description,
    NEW.published_at,
    NEW.published_by
  );
END;

CREATE TRIGGER IF NOT EXISTS galleries_history_au_end
AFTER UPDATE OF removed_at ON galleries_history
BEGIN
  DELETE FROM galleries
  WHERE gallery_id = OLD.gallery_id;
END;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
