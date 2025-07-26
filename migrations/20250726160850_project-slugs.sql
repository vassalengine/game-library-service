COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tmp (
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

INSERT INTO tmp SELECT project_id, name, normalized_name, name AS slug, created_at, modified_at, modified_by, revision, description, game_title, game_title_sort, game_publisher, game_year, game_players_min, game_players_max, game_length_min, game_length_max, readme, image FROM projects;

DROP TABLE projects_fts;
DROP TRIGGER projects_ai;
DROP TRIGGER projects_ad;
DROP TRIGGER projects_au;

DROP TABLE projects;
ALTER TABLE tmp RENAME TO projects;

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

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
