COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
  gallery_id INTEGER PRIMARY KEY NOT NULL CHECK(gallery_id >= 0),
  prev_id INTEGER REFERENCES galleries_history(gallery_id),
  next_id INTEGER REFERENCES galleries_history(gallery_id),
  sort_key BLOB,
  project_id INTEGER NOT NULL,
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
  UNIQUE(project_id, filename),
  CHECK(next_id != gallery_id),
  CHECK(prev_id != gallery_id),
  CHECK(
    (removed_at IS NULL AND removed_by IS NULL) OR
    (removed_at >= published_at AND removed_by IS NOT NULL)
  )
);

INSERT INTO tmp SELECT gallery_id, prev_id, next_id, NULL, project_id, filename, description, published_at, published_by, removed_at, removed_by FROM galleries_history;

DROP TABLE galleries_history;
ALTER TABLE tmp RENAME TO galleries_history;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
