CREATE TABLE IF NOT EXISTS packages_history (
  package_id INTEGER PRIMARY KEY NOT NULL CHECK(package_id >= 0),
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  created_by INTEGER NOT NULL,
  deleted_at INTEGER,
  deleted_by INTEGER,
  FOREIGN KEY(project_id) REFERENCES projects(project_id),
  FOREIGN KEY(created_by) REFERENCES users(user_id),
  FOREIGN KEY(deleted_by) REFERENCES users(user_id),
  CHECK(
    (deleted_at IS NULL AND deleted_by IS NULL) OR
    (deleted_at IS NOT NULL AND deleted_by IS NOT NULL)
  ),
  CHECK(deleted_at IS NULL OR created_at <= deleted_at)
);
