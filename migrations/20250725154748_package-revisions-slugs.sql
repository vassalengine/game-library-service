COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
  package_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  slug TEXT NOT NULL,
  sort_key INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(modified_by) REFERENCES users(user_id)
);

INSERT INTO tmp SELECT package_id, name, name AS slug, sort_key, modified_at, modified_by FROM packages_revisions;

DROP TABLE packages_revisions;
ALTER TABLE tmp RENAME TO packages_revisions;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
