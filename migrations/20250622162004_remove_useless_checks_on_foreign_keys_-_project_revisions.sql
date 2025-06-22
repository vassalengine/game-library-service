PRAGMA foreign_keys = OFF;

CREATE TABLE tmp (
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL,
  revision INTEGER NOT NULL CHECK(revision >= 0),
  project_data_id INTEGER NOT NULL,
  UNIQUE(project_id, revision),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(modified_by) REFERENCES users(user_id),
  FOREIGN KEY(project_data_id) REFERENCES project_data(project_data_id)
);

INSERT INTO tmp SELECT * FROM project_revisions;
DROP TABLE project_revisions;
ALTER TABLE tmp RENAME TO project_revisions;

PRAGMA foreign_keys = ON;
