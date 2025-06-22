PRAGMA foreign_keys = OFF;

CREATE TABLE tmp(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

INSERT INTO tmp SELECT * FROM players;
DROP TABLE players;
ALTER TABLE tmp RENAME TO players;

PRAGMA foreign_keys = ON;
