/* TODO: check queries for what indices we need */

CREATE TABLE IF NOT EXISTS users (
  user_id INTEGER PRIMARY KEY NOT NULL,
  username TEXT NOT NULL,
  UNIQUE(username)
);

CREATE TABLE IF NOT EXISTS owners (
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE IF NOT EXISTS players (
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(user_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE IF NOT EXISTS files (
  file_id INTEGER PRIMARY KEY NOT NULL CHECK(file_id >= 0),
  release_id INTEGER NOT NULL,
  url TEXT NOT NULL,
  filename TEXT NOT NULL,
  size INTEGER NOT NULL CHECK(size >= 0),
  sha256 TEXT NOT NULL,
  content_type TEXT NOT NULL,
  requires TEXT,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  UNIQUE(release_id, filename),
  FOREIGN KEY(release_id) REFERENCES releases(release_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS releases_history (
  release_id INTEGER PRIMARY KEY NOT NULL CHECK(release_id >= 0),
  package_id INTEGER NOT NULL,
  version TEXT NOT NULL,
  version_major INTEGER NOT NULL CHECK(version_major >= 0),
  version_minor INTEGER NOT NULL CHECK(version_minor >= 0),
  version_patch INTEGER NOT NULL CHECK(version_patch >= 0),
  version_pre TEXT NOT NULL,
  version_build TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  deleted_at INTEGER,
  deleted_by INTEGER,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(deleted_by) REFERENCES users(user_id)
  CHECK(
    (deleted_at IS NULL AND deleted_by IS NULL) OR
    (deleted_at >= published_at AND deleted_by IS NOT NULL)
  )
);

CREATE TABLE IF NOT EXISTS releases (
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

CREATE TABLE IF NOT EXISTS packages_history (
  package_id INTEGER PRIMARY KEY NOT NULL CHECK(package_id >= 0),
  project_id INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  deleted_at INTEGER,
  deleted_by INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  FOREIGN KEY(deleted_by) REFERENCES users(user_id),
  CHECK(
    (deleted_at IS NULL AND deleted_by IS NULL) OR
    (deleted_at >= created_at AND deleted_by IS NOT NULL)
  )
);

CREATE TABLE IF NOT EXISTS packages_revisions (
  package_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  slug TEXT NOT NULL,
  sort_key INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(modified_by) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS packages (
  package_id INTEGER PRIMARY KEY NOT NULL, 
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  slug TEXT NOT NULL,
  sort_key INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  FOREIGN KEY(package_id) REFERENCES packages_history(package_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  UNIQUE(project_id, name),
  UNIQUE(project_id, slug),
  UNIQUE(project_id, sort_key)
);

CREATE TABLE IF NOT EXISTS images (
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  content_type TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename)
);

CREATE TABLE IF NOT EXISTS image_revisions (
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  content_type TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  UNIQUE(project_id, filename, published_at)
);

CREATE TABLE IF NOT EXISTS galleries_history (
  gallery_id INTEGER PRIMARY KEY NOT NULL CHECK(gallery_id >= 0),
  prev_id INTEGER REFERENCES galleries_history(gallery_id),
  next_id INTEGER REFERENCES galleries_history(gallery_id),
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  removed_at INTEGER,
  removed_by INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(removed_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(project_id, filename),
  CHECK(next_id != gallery_id),
  CHECK(prev_id != gallery_id),
  CHECK(
    (removed_at IS NULL AND removed_by IS NULL) OR
    (removed_at >= published_at AND removed_by IS NOT NULL)
  )
);

CREATE TABLE IF NOT EXISTS galleries (
  gallery_id INTEGER PRIMARY KEY NOT NULL,
  prev_id INTEGER REFERENCES galleries(gallery_id),
  next_id INTEGER REFERENCES galleries(gallery_id),
  project_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  description TEXT NOT NULL,
  published_at INTEGER NOT NULL,
  published_by INTEGER NOT NULL,
  FOREIGN KEY(gallery_id) REFERENCES galleries_history(gallery_id),
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(published_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, filename) REFERENCES images(project_id, filename),
  UNIQUE(prev_id),
  UNIQUE(next_id),
  UNIQUE(project_id, filename),
  CHECK(prev_id != gallery_id),
  CHECK(next_id != gallery_id)
);

CREATE TABLE IF NOT EXISTS tags (
  project_id INTEGER NOT NULL,
  tag TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  UNIQUE(project_id, tag)
);

CREATE TABLE IF NOT EXISTS projects_history (
  project_id INTEGER PRIMARY KEY NOT NULL CHECK(project_id >= 0),
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS projects_data (
  project_data_id INTEGER PRIMARY KEY NOT NULL CHECK(project_data_id >= 0),
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  slug TEXT NOT NULL,
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
  FOREIGN KEY(project_id) REFERENCES projects_history(project_id),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename)
);

CREATE TABLE IF NOT EXISTS projects_revisions (
  project_id INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL,
  revision INTEGER NOT NULL CHECK(revision >= 0),
  project_data_id INTEGER NOT NULL,
  UNIQUE(project_id, revision),
  FOREIGN KEY(project_id) REFERENCES projects_history(project_id),
  FOREIGN KEY(modified_by) REFERENCES users(user_id),
  FOREIGN KEY(project_data_id) REFERENCES projects_data(project_data_id)
);

CREATE TABLE IF NOT EXISTS projects (
  project_id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
  slug TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  modified_at INTEGER NOT NULL,
  modified_by INTEGER NOT NULL,
  revision INTEGER NOT NULL,
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
  UNIQUE(slug),
  FOREIGN KEY(project_id) REFERENCES projects_history(project_id),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename),
  FOREIGN KEY(modified_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, revision) REFERENCES projects_revisions(project_id, revision)
);

CREATE TABLE IF NOT EXISTS flags (
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

/* Full-text search */

CREATE VIRTUAL TABLE IF NOT EXISTS projects_fts USING fts5(
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

CREATE TRIGGER IF NOT EXISTS projects_ai AFTER INSERT ON projects
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
    NEW.project_id,
    NEW.game_title,
    NEW.game_publisher,
    NEW.game_year,
    NEW.description,
    NEW.readme
  );
END;

CREATE TRIGGER IF NOT EXISTS projects_ad AFTER DELETE ON projects
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
    OLD.project_id,
    OLD.game_title,
    OLD.game_publisher,
    OLD.game_year,
    OLD.description,
    OLD.readme
  );
END;

CREATE TRIGGER IF NOT EXISTS projects_au AFTER UPDATE ON projects
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
    OLD.project_id,
    OLD.game_title,
    OLD.game_publisher,
    OLD.game_year,
    OLD.description,
    OLD.readme
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
    NEW.project_id,
    NEW.game_title,
    NEW.game_publisher,
    NEW.game_year,
    NEW.description,
    NEW.readme
  );
END;
