COMMIT TRANSACTION;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

INSERT INTO packages_history SELECT *, NULL, NULL FROM packages;

CREATE TABLE tmp (
  package_id INTEGER PRIMARY KEY NOT NULL, 
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  UNIQUE(project_id, name)
);

INSERT INTO tmp SELECT * FROM packages;
DROP TABLE packages;
ALTER TABLE tmp RENAME TO packages;

COMMIT TRANSACTION;

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;
