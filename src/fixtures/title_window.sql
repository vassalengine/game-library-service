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
  (1, "a", "", "", "", "a", "", "", "", NULL, "", 1, 1),
  (2, "b", "", "", "", "a", "", "", "", NULL, "", 1, 1),
  (3, "c", "", "", "", "b", "", "", "", NULL, "", 1, 1),
  (4, "d", "", "", "", "c", "", "", "", NULL, "", 1, 1);
