DROP TABLE projects;

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
