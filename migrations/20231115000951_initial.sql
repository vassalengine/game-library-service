CREATE TABLE users(
  user_id INTEGER PRIMARY KEY NOT NULL,
  username TEXT NOT NULL,
  UNIQUE(username)
);

CREATE TABLE owners(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE authors(
  user_id INTEGER NOT NULL,
  release_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(release_id) REFERENCES releases(release_id),
  UNIQUE(user_id, release_id)
);

CREATE TABLE players(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE packages (
  package_id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id)
);

CREATE TABLE releases (
  release_id INTEGER PRIMARY KEY NOT NULL,
  package_id INTEGER NOT NULL,
  version TEXT NOT NULL,
  version_major INTEGER NOT NULL,
  version_minor INTEGER NOT NULL,
  version_patch INTEGER NOT NULL,
  version_pre TEXT NOT NULL,
  version_build TEXT NOT NULL,
  url TEXT NOT NULL,
  filename TEXT NOT NULL,
  size INTEGER NOT NULL,
  checksum TEXT NOT NULL,
  published_at TEXT NOT NULL,
  published_by INTEGER NOT NULL,
  UNIQUE(package_id, version_major, version_minor, version_patch),
  FOREIGN KEY(package_id) REFERENCES packages(package_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id)
);

CREATE TABLE images (
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  published_at TEXT NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename)
);

CREATE TABLE projects (
  project_id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,

  /* project data */
  description TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,

  /* readme */
  readme TEXT NOT NULL,

  /* image */
  image TEXT,

  /* project revision */
  modified_at TEXT NOT NULL,
  revision INTEGER NOT NULL,

  UNIQUE(name),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename)
);

CREATE TABLE projects_arch (
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,

  /* project data */
  description TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,

  /* readme */
  readme TEXT NOT NULL,

  /* image */
  image TEXT,

  /* project revision */
  modified_at TEXT NOT NULL,
  revision INTEGER NOT NULL,

  UNIQUE(project_id, revision),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename)
);

/* Full-text search */

CREATE VIRTUAL TABLE projects_fts USING fts5(
  description,
  game_title,
  game_publisher,
  game_year,
  readme,
  content="projects",
  content_rowid="project_id"
);

CREATE TRIGGER projects_ai AFTER INSERT ON projects
BEGIN
  INSERT INTO projects_fts (
    rowid,
    description,
    game_title,
    game_publisher,
    game_year,
    readme
  )
  VALUES (
    new.project_id,
    new.description,
    new.game_title,
    new.game_publisher,
    new.game_year,
    new.readme
  );
END;

CREATE TRIGGER projects_ad AFTER DELETE ON projects
BEGIN
  INSERT INTO projects_fts (
    projects_fts,
    rowid,
    description,
    game_title,
    game_publisher,
    game_year,
    readme
  )
  VALUES (
    'delete',
    old.project_id,
    old.description,
    old.game_title,
    old.game_publisher,
    old.game_year,
    old.readme
  );
END;

CREATE TRIGGER projects_au AFTER UPDATE ON projects
BEGIN
  INSERT INTO projects_fts (
    projects_fts,
    rowid,
    description,
    game_title,
    game_publisher,
    game_year,
    readme
  )
  VALUES (
    'delete',
    old.project_id,
    old.description,
    old.game_title,
    old.game_publisher,
    old.game_year,
    old.readme
  );
  INSERT INTO projects_fts (
    rowid,
    description,
    game_title,
    game_publisher,
    game_year,
    readme
  )
  VALUES (
    new.project_id,
    new.description,
    new.game_title,
    new.game_publisher,
    new.game_year,
    new.readme
  );
END;

/* SELECT rowid, * FROM projects_fts WHERE projects_fts MATCH 'Afrika' ORDER BY rank; */
