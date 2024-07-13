INSERT INTO projects (
  project_id,
  name,
  normalized_name,
  created_at,
  description,
  game_title,
  game_title_sort,
  game_publisher,
  game_year,
  game_players_min,
  game_players_max,
  game_length_min,
  game_length_max,
  readme,
  image,
  modified_at,
  modified_by,
  revision
)
VALUES
  (1, "a", "a", 0, "abc xyz", "", "", "", "", NULL, NULL, NULL, NULL, "", NULL, 0, 1, 1),
  (2, "b", "b", 0, "pdq", "", "", "", "", NULL, NULL, NULL, NULL, "", NULL, 0, 1, 1),
  (3, "c", "c", 0, "abc", "", "", "", "", NULL, NULL, NULL, NULL, "", NULL, 0, 1, 1),
  (4, "d", "d", 0, "abc", "", "", "", "", NULL, NULL, NULL, NULL, "", NULL, 0, 1, 1);
