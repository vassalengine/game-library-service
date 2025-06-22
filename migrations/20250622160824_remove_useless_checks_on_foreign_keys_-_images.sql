PRAGMA foreign_keys = OFF;

CREATE TABLE tmp (
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename)
);

INSERT INTO tmp SELECT * FROM images;
DROP TABLE images;
ALTER TABLE tmp RENAME TO images;

PRAGMA foreign_keys = ON;
