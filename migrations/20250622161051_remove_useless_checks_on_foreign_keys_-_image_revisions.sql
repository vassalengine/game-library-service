PRAGMA foreign_keys = OFF;

CREATE TABLE tmp (
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename, published_at)
);

INSERT INTO tmp SELECT * FROM image_revisions;
DROP TABLE image_revisions;
ALTER TABLE tmp RENAME TO image_revisions;

PRAGMA foreign_keys = ON;
