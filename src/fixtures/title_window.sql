INSERT INTO projects (
  project_id,
  name,
  created_at,
  description,
  game_title,
  game_title_sort,
  game_publisher,
  game_year,
  readme,
  image,
  modified_at,
  modified_by,
  revision
)
VALUES
  (1, "a", 0, "", "", "a", "", "", "", NULL, 1, 1, 1),
  (2, "b", 0, "", "", "a", "", "", "", NULL, 2, 1, 1),
  (3, "c", 0, "", "", "b", "", "", "", NULL, 3, 1, 1),
  (4, "d", 0, "", "", "c", "", "", "", NULL, 4, 1, 1);
