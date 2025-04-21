/* TODO: add indices */

CREATE TABLE users(
  user_id INTEGER PRIMARY KEY NOT NULL CHECK(user_id >= 0),
  username TEXT NOT NULL,
  UNIQUE(username)
);

CREATE TABLE owners(
  user_id INTEGER NOT NULL CHECK(user_id >= 0),
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE authors(
  user_id INTEGER NOT NULL CHECK(user_id >= 0),
  release_id INTEGER NOT NULL CHECK(release_id >= 0),
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(release_id) REFERENCES releases(release_id),
  UNIQUE(user_id, release_id)
);

CREATE TABLE players(
  user_id INTEGER NOT NULL CHECK(user_id >= 0),
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE packages (
  package_id INTEGER PRIMARY KEY NOT NULL CHECK(package_id >= 0),
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  UNIQUE(project_id, name)
);

CREATE TABLE releases (
  release_id INTEGER PRIMARY KEY NOT NULL CHECK(release_id >= 0),
  package_id INTEGER NOT NULL CHECK(release_id >= 0),
  version TEXT NOT NULL,
  version_major INTEGER NOT NULL CHECK(version_major >= 0),
  version_minor INTEGER NOT NULL CHECK(version_minor >= 0),
  version_patch INTEGER NOT NULL CHECK(version_patch >= 0),
  version_pre TEXT NOT NULL,
  version_build TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  UNIQUE(package_id, version_major, version_minor, version_patch, version_pre, version_build),
  FOREIGN KEY(package_id) REFERENCES packages(package_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id)
);

CREATE TABLE files (
  file_id INTEGER PRIMARY KEY NOT NULL CHECK(file_id >= 0),
  release_id INTEGER NOT NULL CHECK(release_id >= 0),
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

CREATE TABLE images (
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename)
);

CREATE TABLE image_revisions (
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename, published_at)
);

CREATE TABLE galleries (
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL CHECK(published_by >= 0),
  removed_at INTEGER,
  removed_by INTEGER CHECK(removed_by >= 0 OR project_id IS NULL),
  position INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(removed_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(project_id, filename),
  UNIQUE(project_id, position),
  CHECK(
    (removed_at IS NULL AND removed_by IS NULL) OR
    (removed_at IS NOT NULL AND removed_by IS NOT NULL)
  )
);

CREATE TABLE tags (
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  tag TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(project_id, tag)
);

CREATE TABLE projects (
  project_id INTEGER PRIMARY KEY NOT NULL CHECK(project_id >= 0),
  name TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL CHECK(modified_by >= 0),
  revision INTEGER NOT NULL CHECK(revision >= 0),
  description TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,
  game_players_min INTEGER CHECK(game_players_min >= 1 OR game_players_min IS NULL),
  game_players_max INTEGER CHECK(game_players_max >= 1 OR game_players_max IS NULL),
  game_length_min INTEGER CHECK(game_length_min >= 1 OR game_length_min IS NULL),
  game_length_max INTEGER CHECK(game_length_max >= 1 OR game_length_max IS NULL),
  readme TEXT NOT NULL,
  image TEXT,
  CHECK(game_players_max >= game_players_min OR game_players_min IS NULL OR game_players_max IS NULL),
  CHECK(game_length_max >= game_length_min OR game_length_min IS NULL OR game_length_max IS NULL),
  UNIQUE(name),
  UNIQUE(normalized_name),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename),
  FOREIGN KEY(modified_by) REFERENCES users(user_id)
);

CREATE TABLE project_revisions (
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL CHECK(modified_by >= 0),
  revision INTEGER NOT NULL CHECK(revision >= 0),
  project_data_id INTEGER NOT NULL CHECK(project_data_id >= 0),
  UNIQUE(project_id, revision),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(modified_by) REFERENCES users(user_id),
  FOREIGN KEY(project_data_id) REFERENCES project_data(project_data_id)
);

CREATE TABLE project_data (
  project_data_id INTEGER PRIMARY KEY NOT NULL CHECK(project_data_id >= 0),
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  description TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,
  game_players_min INTEGER CHECK(game_players_min >= 1 OR game_players_min IS NULL),
  game_players_max INTEGER CHECK(game_players_max >= 1 OR game_players_max IS NULL),
  game_length_min INTEGER CHECK(game_length_min >= 1 OR game_length_min IS NULL),
  game_length_max INTEGER CHECK(game_length_max >= 1 OR game_length_max IS NULL),
  readme TEXT NOT NULL,
  image TEXT,
  CHECK(game_players_max >= game_players_min OR game_players_min IS NULL OR game_players_max IS NULL),
  CHECK(game_length_max >= game_length_min OR game_length_min IS NULL OR game_length_max IS NULL),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename)
);

CREATE TABLE flags(
  flag_id INTEGER PRIMARY KEY NOT NULL CHECK(flag_id >= 0),
  user_id INTEGER NOT NULL CHECK(user_id >= 0),
  project_id INTEGER NOT NULL CHECK(project_id >= 0),
  flag INTEGER NOT NULL CHECK(flag >= 1 AND flag <= 4),
  message TEXT,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id)
);

/* Full-text search */

CREATE VIRTUAL TABLE projects_fts USING fts5(
  game_title,
  game_publisher,
  game_year,
  description,
  readme,
  content="projects",
  content_rowid="project_id"
);

/* Set weight for game title to 100 */
INSERT INTO projects_fts(
  projects_fts,
  rank
) VALUES(
  'rank',
  'bm25(100.0)'
);

CREATE TRIGGER projects_ai AFTER INSERT ON projects
BEGIN
  INSERT INTO projects_fts (
    rowid,
    game_title,
    game_publisher,
    game_year,
    description,
    readme
  )
  VALUES (
    new.project_id,
    new.game_title,
    new.game_publisher,
    new.game_year,
    new.description,
    new.readme
  );
END;

CREATE TRIGGER projects_ad AFTER DELETE ON projects
BEGIN
  INSERT INTO projects_fts (
    projects_fts,
    rowid,
    game_title,
    game_publisher,
    game_year,
    description,
    readme
  )
  VALUES (
    'delete',
    old.project_id,
    old.game_title,
    old.game_publisher,
    old.game_year,
    old.description,
    old.readme
  );
END;

CREATE TRIGGER projects_au AFTER UPDATE ON projects
BEGIN
  INSERT INTO projects_fts (
    projects_fts,
    rowid,
    game_title,
    game_publisher,
    game_year,
    description,
    readme
  )
  VALUES (
    'delete',
    old.project_id,
    old.game_title,
    old.game_publisher,
    old.game_year,
    old.description,
    old.readme
  );
  INSERT INTO projects_fts (
    rowid,
    game_title,
    game_publisher,
    game_year,
    description,
    readme
  )
  VALUES (
    new.project_id,
    new.game_title,
    new.game_publisher,
    new.game_year,
    new.description,
    new.readme
  );
END;
