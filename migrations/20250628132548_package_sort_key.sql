COMMIT TRANSACTION;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

CREATE TABLE tmp (
  package_id INTEGER PRIMARY KEY NOT NULL CHECK(package_id >= 0),
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  sort_key INTEGER NOT NULL,
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

INSERT INTO tmp SELECT package_id, project_id, name, 0, created_at, created_by, deleted_at, deleted_by FROM packages_history;

UPDATE tmp SET sort_key = q.sort_key FROM (SELECT package_id, row_number() OVER (PARTITION BY project_id GROUPS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS sort_key FROM tmp) AS q WHERE tmp.package_id = q.package_id;

DROP TABLE packages_history;
ALTER TABLE tmp RENAME TO packages_history;

COMMIT TRANSACTION;

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;
