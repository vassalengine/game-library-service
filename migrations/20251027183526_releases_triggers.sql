COMMIT TRANSACTION;
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TRIGGER IF NOT EXISTS releases_history_ai_start
AFTER INSERT ON releases_history
BEGIN
  INSERT INTO releases (
    release_id,
    package_id,
    version,
    version_major,
    version_minor,
    version_patch,
    version_pre,
    version_build,
    published_at,
    published_by
  )
  VALUES (
    NEW.release_id,
    NEW.package_id,
    NEW.version,
    NEW.version_major,
    NEW.version_minor,
    NEW.version_patch,
    NEW.version_pre,
    NEW.version_build,
    NEW.published_at,
    NEW.published_by
  );
END;

CREATE TRIGGER IF NOT EXISTS releases_history_au_end
AFTER UPDATE OF deleted_at ON releases_history
BEGIN
  DELETE FROM releases
  WHERE release_id = OLD.release_id;
END;

COMMIT TRANSACTION;
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
