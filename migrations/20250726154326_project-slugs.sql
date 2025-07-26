COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
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

INSERT INTO tmp SELECT project_data_id, project_id, name, "" AS slug, description, game_title, game_title_sort, game_publisher, game_year, game_players_min, game_players_max, game_length_min, game_length_max, readme, image FROM projects_data;

DROP TABLE projects_data;
ALTER TABLE tmp RENAME TO projects_data;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
