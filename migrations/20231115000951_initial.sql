CREATE TABLE users(
  id INTEGER PRIMARY KEY NOT NULL,
  username TEXT NOT NULL,
  UNIQUE(username)
);

CREATE TABLE owners(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(project_id) REFERENCES projects(id),
  UNIQUE(user_id, project_id)
);

CREATE TABLE players(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(project_id) REFERENCES projects(id)
);

CREATE TABLE readmes (
  project_id INTEGER NOT NULL,
  revision INTEGER NOT NULL,
  text TEXT NOT NULL,
  PRIMARY KEY(project_id, revision),
  FOREIGN KEY(project_id) REFERENCES projects(id)
);

CREATE TABLE packages (
  id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(id)
);

CREATE TABLE package_versions (
  id INTEGER PRIMARY KEY NOT NULL,
  package_id INTEGER NOT NULL,
  version TEXT NOT NULL,
  version_major INTEGER NOT NULL,
  version_minor INTEGER NOT NULL,
  version_patch INTEGER NOT NULL,
  version_pre TEXT,
  version_build TEXT,
  filename TEXT NOT NULL,
  url TEXT NOT NULL,
  UNIQUE(package_id, version_major, version_minor, version_patch),
  FOREIGN KEY(package_id) REFERENCES packages(id)
);

CREATE TABLE projects (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  revision INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  modified_at TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,
  UNIQUE(name)
);

CREATE TABLE projects_revisions (
  id INTEGER NOT NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  revision INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  modified_at TEXT NOT NULL,
  game_title TEXT NOT NULL,
  game_title_sort TEXT NOT NULL,
  game_publisher TEXT NOT NULL,
  game_year TEXT NOT NULL,
  UNIQUE(id, revision)
);
