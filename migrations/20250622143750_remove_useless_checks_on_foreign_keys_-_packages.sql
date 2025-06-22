PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS tmp(
  package_id INTEGER PRIMARY KEY NOT NULL CHECK(package_id >= 0),
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  UNIQUE(project_id, name)
);

INSERT INTO tmp SELECT * FROM packages;
DROP TABLE packages;
ALTER TABLE tmp RENAME TO packages;

PRAGMA foreign_keys = ON;
