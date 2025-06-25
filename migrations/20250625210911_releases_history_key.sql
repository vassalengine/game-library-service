COMMIT TRANSACTION;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

INSERT INTO releases_history SELECT *, NULL, NULL FROM releases;

CREATE TABLE tmp (
  release_id INTEGER PRIMARY KEY NOT NULL,
  package_id INTEGER NOT NULL,
  version TEXT NOT NULL,
  version_major INTEGER NOT NULL CHECK(version_major >= 0),
  version_minor INTEGER NOT NULL CHECK(version_minor >= 0),
  version_patch INTEGER NOT NULL CHECK(version_patch >= 0),
  version_pre TEXT NOT NULL,
  version_build TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  UNIQUE(package_id, version_major, version_minor, version_patch, version_pre, version_build),
  FOREIGN KEY(release_id) REFERENCES releases_history(release_id),
  FOREIGN KEY(package_id) REFERENCES packages(package_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id)
);

INSERT INTO tmp SELECT * FROM releases;
DROP TABLE releases;
ALTER TABLE tmp RENAME TO releases;

COMMIT TRANSACTION;

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;
