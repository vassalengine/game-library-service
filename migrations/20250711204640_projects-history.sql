COMMIT TRANSACTION;

PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS projects_history (
  project_id INTEGER PRIMARY KEY NOT NULL CHECK(project_id >= 0),
  created_at INTEGER NOT NULL
);

INSERT OR IGNORE INTO projects_history SELECT project_id, created_at FROM projects;


CREATE TABLE IF NOT EXISTS projects_data (
  project_data_id INTEGER PRIMARY KEY NOT NULL CHECK(project_data_id >= 0),
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
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

INSERT OR IGNORE INTO projects_data SELECT project_data_id, project_id, "", description, game_title, game_title_sort, game_publisher, game_year, game_players_min, game_players_max, game_length_min, game_length_max, readme, image FROM project_data;

DROP TABLE IF EXISTS project_data;

UPDATE projects_data SET name = projects.name FROM projects WHERE projects_data.project_id = projects.project_id;

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

INSERT OR IGNORE INTO projects_revisions SELECT project_id, modified_at, modified_by, revision, project_data_id FROM project_revisions;

DROP TABLE project_revisions;

CREATE TABLE tmp (
  project_id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  normalized_name TEXT NOT NULL,
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
  FOREIGN KEY(project_id) REFERENCES projects_history(project_id),
  FOREIGN KEY(project_id, image) REFERENCES images(project_id, filename),
  FOREIGN KEY(modified_by) REFERENCES users(user_id),
  FOREIGN KEY(project_id, revision) REFERENCES projects_revisions(project_id, revision)
);

INSERT INTO tmp SELECT * FROM projects;

DROP TABLE projects;
ALTER TABLE tmp RENAME TO projects;

DROP TABLE projects_fts;

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
    new.project_id,
    new.game_title,
    new.game_publisher,
    new.game_year,
    new.description,
    new.readme
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
    old.project_id,
    old.game_title,
    old.game_publisher,
    old.game_year,
    old.description,
    old.readme
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

INSERT INTO projects_fts(projects_fts) VALUES('rebuild');

COMMIT TRANSACTION;

PRAGMA foreign_keys = ON;
PRAGMA foreign_key_check;

BEGIN TRANSACTION;
