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
  (1, "a", 0, "abc xyz", "", "", "", "", "", NULL, 0, 1, 1),
  (2, "b", 0, "pdq", "", "", "", "", "", NULL, 0, 1, 1),
  (3, "c", 0, "abc", "", "", "", "", "", NULL, 0, 1, 1),
  (4, "d", 0, "abc", "", "", "", "", "", NULL, 0, 1, 1);
