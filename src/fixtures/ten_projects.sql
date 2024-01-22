INSERT INTO projects (
  project_id,
  name,
  created_at
)
VALUES
  (1, "a", ""),
  (2, "b", ""),
  (3, "c", ""),
  (4, "d", ""),
  (5, "e", ""),
  (6, "f", ""),
  (7, "g", ""),
  (8, "h", ""),
  (9, "i", ""),
  (10, "j", "");

INSERT INTO project_data (
  project_data_id,
  project_id,
  description,
  game_title,
  game_title_sort,
  game_publisher,
  game_year
)
VALUES
  (1, 1, "", "", "", "", ""),
  (2, 2, "", "", "", "", ""),
  (3, 3, "", "", "", "", ""),
  (4, 4, "", "", "", "", ""),
  (5, 5,"", "", "", "", ""),
  (6, 6, "", "", "", "", ""),
  (7, 7, "", "", "", "", ""),
  (8, 8, "", "", "", "", ""),
  (9, 9, "", "", "", "", ""),
  (10, 10, "", "", "", "", "");

INSERT INTO readmes (
  readme_id,
  project_id,
  text
)
VALUES
  (1, 1, ""),
  (2, 2, ""),
  (3, 3, ""),
  (4, 4, ""),
  (5, 5, ""),
  (6, 6, ""),
  (7, 7, ""),
  (8, 8, ""),
  (9, 9, ""),
  (10, 10, "");


INSERT INTO project_revisions (
  project_id,
  revision,
  project_data_id,
  readme_id,
  modified_at
)
VALUES
  (1, 1, 1, 1, ""),
  (2, 1, 2, 2, ""),
  (3, 1, 3, 3, ""),
  (4, 1, 4, 4, ""),
  (5, 1, 5, 5, ""),
  (6, 1, 6, 6, ""),
  (7, 1, 7, 7, ""),
  (8, 1, 8, 8, ""),
  (9, 1, 9, 9, ""),
  (10, 1, 10, 10, "");
