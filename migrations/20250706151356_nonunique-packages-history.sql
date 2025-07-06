COMMIT TRANSACTION;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

CREATE TABLE tmp (
  package_id INTEGER PRIMARY KEY NOT NULL CHECK(package_id >= 0),
  project_id INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  deleted_at INTEGER,
  deleted_by INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  FOREIGN KEY(deleted_by) REFERENCES users(user_id),
  CHECK(
    (deleted_at IS NULL AND deleted_by IS NULL) OR
    (deleted_at IS NOT NULL AND deleted_by IS NOT NULL)
  ),
  CHECK(deleted_at IS NULL OR created_at <= deleted_at)
);

INSERT INTO tmp SELECT package_id, project_id, created_at, created_by, deleted_at, deleted_by FROM packages_history;

CREATE TABLE IF NOT EXISTS packages_revisions (
  package_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  sort_key INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(modified_by) REFERENCES users(user_id)
);

INSERT INTO packages_revisions SELECT package_id, name, sort_key, created_at AS modified_at, created_by AS modified_by FROM packages_history;

DROP TABLE packages_history;
ALTER TABLE tmp RENAME TO packages_history;

COMMIT TRANSACTION;

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;

