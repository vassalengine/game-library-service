/* Record the VASSAL module name contained in each uploaded .vmod, so that
   projects can be found by the module name a player sees in VASSAL (or in a
   save's moduledata) rather than only by their game metadata. */

/* The module name extracted from a .vmod's moduledata <name> element;
   NULL for non-module files and for files uploaded before this change. */
ALTER TABLE files ADD COLUMN module_name TEXT;

/* The distinct module names of a project's files, newline-joined — derived
   from files.module_name, kept up to date on upload, and existing only to
   feed the full-text index below. */
ALTER TABLE projects ADD COLUMN module_names TEXT NOT NULL DEFAULT '';

/* Recreate the full-text index with the module_names column. An fts5
   external-content table cannot be altered, so drop and rebuild it. */

DROP TRIGGER IF EXISTS projects_ai;
DROP TRIGGER IF EXISTS projects_ad;
DROP TRIGGER IF EXISTS projects_au;
DROP TABLE IF EXISTS projects_fts;

CREATE VIRTUAL TABLE IF NOT EXISTS projects_fts USING fts5(
  game_title,
  game_publisher,
  game_year,
  description,
  readme,
  module_names,
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
    readme,
    module_names
  )
  VALUES (
    NEW.project_id,
    NEW.game_title,
    NEW.game_publisher,
    NEW.game_year,
    NEW.description,
    NEW.readme,
    NEW.module_names
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
    readme,
    module_names
  )
  VALUES (
    'delete',
    OLD.project_id,
    OLD.game_title,
    OLD.game_publisher,
    OLD.game_year,
    OLD.description,
    OLD.readme,
    OLD.module_names
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
    readme,
    module_names
  )
  VALUES (
    'delete',
    OLD.project_id,
    OLD.game_title,
    OLD.game_publisher,
    OLD.game_year,
    OLD.description,
    OLD.readme,
    OLD.module_names
  );
  INSERT INTO projects_fts (
    rowid,
    game_title,
    game_publisher,
    game_year,
    description,
    readme,
    module_names
  )
  VALUES (
    NEW.project_id,
    NEW.game_title,
    NEW.game_publisher,
    NEW.game_year,
    NEW.description,
    NEW.readme,
    NEW.module_names
  );
END;

/* Reindex the existing rows */
INSERT INTO projects_fts(projects_fts) VALUES('rebuild');
