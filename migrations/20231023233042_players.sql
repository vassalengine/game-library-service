CREATE TABLE players(
  user_id INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(project_id) REFERENCES projects(id)
);
