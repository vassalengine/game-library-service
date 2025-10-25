COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
  gallery_id INTEGER PRIMARY KEY NOT NULL,
  prev_id INTEGER REFERENCES galleries(gallery_id),
  next_id INTEGER REFERENCES galleries(gallery_id),
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(gallery_id) REFERENCES galleries_history(gallery_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(prev_id),
  UNIQUE(next_id),
  UNIQUE(project_id, filename),
  CHECK(prev_id != gallery_id),
  CHECK(next_id != gallery_id)
);

INSERT INTO tmp SELECT * FROM galleries;

DROP TABLE galleries;
ALTER TABLE tmp RENAME TO galleries;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
