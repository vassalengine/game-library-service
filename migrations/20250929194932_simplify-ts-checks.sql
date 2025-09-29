COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
  release_id INTEGER PRIMARY KEY NOT NULL CHECK(release_id >= 0),
  package_id INTEGER NOT NULL,
  version TEXT NOT NULL,
  version_major INTEGER NOT NULL CHECK(version_major >= 0),
  version_minor INTEGER NOT NULL CHECK(version_minor >= 0),
  version_patch INTEGER NOT NULL CHECK(version_patch >= 0),
  version_pre TEXT NOT NULL,
  version_build TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  deleted_at INTEGER,
  deleted_by INTEGER,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(deleted_by) REFERENCES users(user_id)
  CHECK(
    (deleted_at IS NULL AND deleted_by IS NULL) OR
    (deleted_at >= published_at AND deleted_by IS NOT NULL)
  )
);

INSERT INTO tmp SELECT * FROM releases_history;

DROP TABLE releases_history;
ALTER TABLE tmp RENAME TO releases_history;

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
    (deleted_at >= created_at AND deleted_by IS NOT NULL)
  )
);

INSERT INTO tmp SELECT * FROM packages_history;

DROP TABLE packages_history;
ALTER TABLE tmp RENAME TO packages_history;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
