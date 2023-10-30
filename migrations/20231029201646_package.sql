CREATE TABLE packages (
  id INTEGER PRIMARY KEY NOT NULL,
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(id)
);

CREATE TABLE package_versions (
  id INTEGER PRIMARY KEY NOT NULL,
  package_id INTEGER NOT NULL,
  version_major INT NOT NULL,
  version_minor INT NOT NULL,
  version_patch INT NOT NULL,
  url TEXT NOT NULL,
  UNIQUE(package_id, version_major, version_minor, version_patch),
  FOREIGN KEY(package_id) REFERENCES packages(id)
);
