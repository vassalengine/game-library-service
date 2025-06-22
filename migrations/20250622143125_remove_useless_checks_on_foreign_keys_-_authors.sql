COMMIT TRANSACTION;

PRAGMA defer_foreign_keys = ON;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS tmp (
  user_id INTEGER NOT NULL,
  release_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(release_id) REFERENCES releases(release_id),
  UNIQUE(user_id, release_id)
);

INSERT INTO tmp SELECT * FROM authors;
DROP TABLE authors;
ALTER TABLE tmp RENAME TO authors;

COMMIT TRANSACTION;

PRAGMA defer_foreign_keys = OFF;

BEGIN TRANSACTION;
