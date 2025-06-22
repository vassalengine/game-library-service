PRAGMA foreign_keys = OFF;

CREATE TABLE tmp (
  file_id INTEGER PRIMARY KEY NOT NULL CHECK(file_id >= 0),
  release_id INTEGER NOT NULL,
  url TEXT NOT NULL,
  filename TEXT NOT NULL,
  size INTEGER NOT NULL CHECK(size >= 0),
  sha256 TEXT NOT NULL,
  requires TEXT,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  UNIQUE(release_id, filename),
  FOREIGN KEY(release_id) REFERENCES releases(release_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id)
);

INSERT INTO tmp SELECT * FROM files;
DROP TABLE files;
ALTER TABLE tmp RENAME TO files;

PRAGMA foreign_keys = ON;
