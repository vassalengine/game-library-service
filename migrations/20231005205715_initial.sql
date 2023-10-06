CREATE TABLE projects (id INTEGER PRIMARY KEY NOT NULL);

CREATE TABLE users(
  id INTEGER PRIMARY KEY NOT NULL,
  username TEXT NOT NULL,
  UNIQUE(username)
);

CREATE TABLE owners(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(project_id) REFERENCES projects(id)
);
