COMMIT TRANSACTION;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

CREATE TABLE tmp (
  package_id INTEGER PRIMARY KEY NOT NULL, 
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  sort_key INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  UNIQUE(project_id, name),
  UNIQUE(project_id, sort_key)
);

INSERT INTO tmp SELECT package_id, project_id, name, row_number() OVER (PARTITION BY project_id GROUPS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS sort_key, created_at, created_by FROM packages; 

DROP TABLE packages;
ALTER TABLE tmp RENAME TO packages;

COMMIT TRANSACTION;

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;
