COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
  flag_id INTEGER PRIMARY KEY NOT NULL CHECK(flag_id >= 0),
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  flagged_at INTEGER NOT NULL,
  closed_at INTEGER,
  closed_by INTEGER,
  flag INTEGER NOT NULL CHECK(flag >= 0 AND flag <= 3),
  message TEXT,
  CHECK(((flag == 0 OR flag == 1) AND message IS NULL) OR ((flag == 2 OR flag == 3) AND message IS NOT NULL)),
  CHECK(
    (closed_at IS NULL AND closed_by IS NULL) OR
    (closed_at >= flagged_at AND closed_by IS NOT NULL)
  ),
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(closed_by) REFERENCES users(user_id)
);

INSERT INTO tmp SELECT flag_id, user_id, project_id, flagged_at, closed_at, iif(closed_at IS NULL, NULL, 5), flag, message FROM flags;

DROP TABLE flags;
ALTER TABLE tmp RENAME TO flags;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
