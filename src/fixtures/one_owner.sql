INSERT INTO users (id, username)
VALUES
  (1, "bob"),
  (2, "alice"),
  (3, "chuck");

INSERT INTO owners (user_id, project_id)
VALUES (1, 42);