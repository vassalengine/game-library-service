INSERT INTO users (id, username)
VALUES
  (1, "bob"),
  (2, "alice"),
  (3, "chuck");

INSERT INTO owners (user_id, project_id)
VALUES
  (1, 6),
  (2, 6),
  (1, 42),
  (2, 42);
