COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

DROP TRIGGER IF EXISTS galleries_history_ai_start;
DROP TRIGGER IF EXISTS galleries_history_au_end;

CREATE TABLE tmp (
  gallery_id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  sort_key BLOB NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(gallery_id) REFERENCES galleries_history(gallery_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(project_id, filename),
  UNIQUE(project_id, sort_key)
);

INSERT INTO tmp SELECT gallery_id, project_id, sort_key, filename, description, published_at, published_by FROM galleries_history;

DROP TABLE galleries;
ALTER TABLE tmp RENAME TO galleries;

CREATE TABLE tmp (
  gallery_id INTEGER PRIMARY KEY NOT NULL CHECK(gallery_id >= 0),
  project_id INTEGER NOT NULL,
  sort_key BLOB NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  removed_at INTEGER,
  removed_by INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(removed_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  CHECK(
    (removed_at IS NULL AND removed_by IS NULL) OR
    (removed_at >= published_at AND removed_by IS NOT NULL)
  )
);

INSERT INTO tmp SELECT gallery_id, project_id, sort_key, filename, description, published_at, published_by, removed_at, removed_by FROM galleries_history;

DROP TABLE galleries_history;
ALTER TABLE tmp RENAME TO galleries_history;

CREATE TRIGGER IF NOT EXISTS galleries_history_ai_start
AFTER INSERT ON galleries_history
BEGIN
  INSERT INTO galleries (
    gallery_id,
    project_id,
    sort_key,
    filename,
    description,
    published_at,
    published_by
  )
  VALUES (
    NEW.gallery_id,
    NEW.project_id,
    NEW.sort_key,
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
