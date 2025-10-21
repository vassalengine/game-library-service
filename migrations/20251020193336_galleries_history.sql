COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS galleries_history (
  gallery_id INTEGER PRIMARY KEY NOT NULL CHECK(gallery_id >= 0),
  prev_id INTEGER REFERENCES galleries_history(gallery_id),
  next_id INTEGER REFERENCES galleries_history(gallery_id),
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  removed_at INTEGER,
  removed_by INTEGER,
  position INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(removed_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(project_id, filename),
  UNIQUE(project_id, position),
  CHECK(next_id != gallery_id),
  CHECK(prev_id != gallery_id),
  CHECK(
    (removed_at IS NULL AND removed_by IS NULL) OR
    (removed_at >= published_at AND removed_by IS NOT NULL)
  )
);

INSERT INTO galleries_history SELECT NULL AS gallery_id, NULL AS prev_id, NULL AS next_id, project_id, filename, description, published_at, published_by, removed_at, removed_by, position FROM galleries;

CREATE TABLE tmp (
  gallery_id INTEGER PRIMARY KEY NOT NULL,
  prev_id INTEGER REFERENCES galleries(gallery_id),
  next_id INTEGER REFERENCES galleries(gallery_id),
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  position INTEGER NOT NULL,
  FOREIGN KEY(gallery_id) REFERENCES galleries_history(gallery_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(project_id, filename),
  UNIQUE(project_id, position),
  CHECK(prev_id != gallery_id),
  CHECK(next_id != gallery_id)
);

INSERT INTO tmp SELECT gallery_id, prev_id, next_id, project_id, filename, description, published_at, published_by, position FROM galleries_history;

DROP TABLE galleries;
ALTER TABLE tmp RENAME TO galleries;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
