PRAGMA foreign_keys = OFF;

CREATE TABLE tmp (
  flag_id INTEGER PRIMARY KEY NOT NULL CHECK(flag_id >= 0),
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  flagged_at INTEGER NOT NULL,
  flag INTEGER NOT NULL CHECK(flag >= 0 AND flag <= 3),
  message TEXT,
  CHECK(((flag == 0 OR flag == 1) AND message IS NULL) OR ((flag == 2 OR flag == 3) AND message IS NOT NULL)),
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id)
);

INSERT INTO tmp SELECT * FROM flags;
DROP TABLE flags;
ALTER TABLE tmp RENAME TO flags;

PRAGMA foreign_keys = ON;
