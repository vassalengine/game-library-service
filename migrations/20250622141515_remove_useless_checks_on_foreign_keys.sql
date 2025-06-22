COMMIT TRANSACTION;

PRAGMA delay_foreign_keys = ON;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS tmp(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

INSERT INTO tmp SELECT * FROM owners;
DROP TABLE owners;
ALTER TABLE tmp RENAME TO owners;

COMMIT TRANSACTION;

PRAGMA delay_foreign_keys = OFF;

BEGIN TRANSACTION;
