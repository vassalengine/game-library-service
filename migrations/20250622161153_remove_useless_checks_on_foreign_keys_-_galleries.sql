COMMIT TRANSACTION;

PRAGMA defer_foreign_keys = ON;

BEGIN TRANSACTION;

CREATE TABLE tmp (
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
  CHECK(
    (removed_at IS NULL AND removed_by IS NULL) OR
    (removed_at IS NOT NULL AND removed_by IS NOT NULL)
  )
);

INSERT INTO tmp SELECT * FROM galleries;
DROP TABLE galleries;
ALTER TABLE tmp RENAME TO galleries;

COMMIT TRANSACTION;

PRAGMA defer_foreign_keys = OFF;

BEGIN TRANSACTION;
