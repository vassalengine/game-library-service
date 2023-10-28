CREATE TABLE readmes (
  project_id INTEGER NOT NULL,
  revision INTEGER NOT NULL,
  text TEXT NOT NULL,
  PRIMARY KEY(project_id, revision),
  FOREIGN KEY(project_id) REFERENCES projects(id)
);
