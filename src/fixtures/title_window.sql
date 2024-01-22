INSERT INTO projects (
  project_id,
  name,
  created_at
)
VALUES
  (1, "a", ""),
  (2, "b", ""),
  (3, "c", ""),
  (4, "d", "");

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
  (1, 1, "", "", "a", "", ""),
  (2, 2, "", "", "a", "", ""),
  (3, 3, "", "", "b", "", ""),
  (4, 4, "", "", "c", "", "");

INSERT INTO project_revisions (
  project_id,
  revision,
  project_data_id,
  readme_id,
  modified_at
)
VALUES
  (1, 1, 1, 0, ""),
  (2, 1, 2, 0, ""),
  (3, 1, 3, 0, ""),
  (4, 1, 4, 0, "");
