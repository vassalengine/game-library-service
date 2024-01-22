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

CREATE TABLE readmes (
  readme_id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  text TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id)
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

CREATE TABLE projects (
  project_id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(name)
);

CREATE TABLE project_data (
  project_data_id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  description TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id)
);

CREATE TABLE project_revisions (
  project_id INTEGER NOT NULL,
  revision INTEGER NOT NULL,
  project_data_id INTEGER NOT NULL,
  readme_id INTEGER NOT NULL,
  image TEXT,
  modified_at TEXT NOT NULL,
  UNIQUE(project_id, revision),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(project_data_id) REFERENCES project_data(project_data_id),
  FOREIGN KEY(readme_id) REFERENCES readmes(readme_id),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename)
);

/* Full-text search */

CREATE VIRTUAL TABLE project_data_fts USING fts5(
  description,
  game_title,
  game_publisher,
  game_year,
  content="project_data",
  content_rowid="project_id"
);

CREATE TRIGGER project_data_ai AFTER INSERT ON project_data
BEGIN
  INSERT INTO project_data_fts (
    rowid,
    description,
    game_title,
    game_publisher,
    game_year
  )
  VALUES (
    new.project_id,
    new.description,
    new.game_title,
    new.game_publisher,
    new.game_year
  );
END;

CREATE TRIGGER project_data_ad AFTER DELETE ON project_data
BEGIN
  INSERT INTO project_data_fts (
    project_data_fts,
    rowid,
    description,
    game_title,
    game_publisher,
    game_year
  )
  VALUES (
    'delete',
    old.project_id,
    old.description,
    old.game_title,
    old.game_publisher,
    old.game_year
  );
END;

CREATE TRIGGER project_data_au AFTER UPDATE ON project_data
BEGIN
  INSERT INTO project_data_fts (
    project_data_fts,
    rowid,
    description,
    game_title,
    game_publisher,
    game_year
  )
  VALUES (
    'delete',
    old.project_id,
    old.description,
    old.game_title,
    old.game_publisher,
    old.game_year
  );
  INSERT INTO project_data_fts (
    rowid,
    description,
    game_title,
    game_publisher,
    game_year
  )
  VALUES (
    new.project_id,
    new.description,
    new.game_title,
    new.game_publisher,
    new.game_year
  );
END;

/* SELECT rowid, * FROM project_data_fts WHERE project_data_fts MATCH 'Afrika' ORDER BY rank; */

CREATE VIRTUAL TABLE readmes_fts USING fts5(
  text,
  content="readmes",
  content_rowid="project_id"
);

CREATE TRIGGER readmes_ai AFTER INSERT ON readmes
BEGIN
  INSERT INTO readmes_fts (
    rowid,
    text
  )
  VALUES (
    new.project_id,
    new.text
  );
END;

CREATE TRIGGER readmes_ad AFTER DELETE ON readmes
BEGIN
  INSERT INTO readmes_fts (
    readmes_fts,
    rowid,
    text
  )
  VALUES (
    'delete',
    old.project_id,
    old.text
  );
END;

CREATE TRIGGER readmes_au AFTER UPDATE ON readmes
BEGIN
  INSERT INTO readmes_fts (
    readmes_fts,
    rowid,
    text
  )
  VALUES (
    'delete',
    old.project_id,
    old.text
  );
  INSERT INTO readmes_fts (
    rowid,
    text
  )
  VALUES (
    new.project_id,
    new.text
  );
END;
