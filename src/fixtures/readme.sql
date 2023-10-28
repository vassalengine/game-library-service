INSERT INTO projects (id, name)
VALUES (42, "some_game");

INSERT INTO readmes (project_id, revision, text)
VALUES
  (42, 1, "first try"),
  (42, 2, "second try"),
  (42, 3, "third try");
