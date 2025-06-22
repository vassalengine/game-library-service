PRAGMA foreign_keys = OFF;

CREATE TABLE tmp (
  project_id INTEGER NOT NULL,
  tag TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(project_id, tag)
);

INSERT INTO tmp SELECT * FROM tags;
DROP TABLE tags;
ALTER TABLE tmp RENAME TO tags;

PRAGMA foreign_keys = ON;
