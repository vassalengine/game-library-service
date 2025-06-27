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
  (
    42,
    "test_game",
    "test_game",
    1699804206419538067,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "game of tests, a",
    "Test Game Company",
    "1979",
    NULL,
    3,
    NULL,
    NULL,
    "",
    NULL,
    1702569006419538067,
    1,
    3
  ),
  (
    6,
    "a_game",
    "a_game",
    1573573806419538067,
    "Another game",
    "Some Other Game",
    "some other game",
    "XYZ",
    "1993",
    NULL,
    NULL,
    NULL,
    NULL,
    "",
    NULL,
    1573573806419538067,
    1,
    1
  );

INSERT INTO project_data (
  project_data_id,
  project_id,
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
  image
)
VALUES
  (
    1,
    42,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "game of tests, a",
    "Test Game Company",
    "1978",
    NULL,
    3,
    NULL,
    NULL,
    "",
    NULL
  ),
  (
    2,
    42,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "game of tests, a",
    "Test Game Company",
    "1979",
    NULL,
    3,
    NULL,
    NULL,
    "",
    NULL
  ),
  (
    3,
    6,
    "Another game",
    "Some Other Game",
    "some other game",
    "XYZ",
    "1993",
    NULL,
    NULL,
    NULL,
    NULL,
    "",
    NULL
  );
;

INSERT INTO project_revisions (
  project_id,
  name,
  created_at,
  modified_at,
  modified_by,
  revision,
  project_data_id
)
VALUES
  (
    42,
    "test_game",
    1699804206419538067,
    1699804206419538067,
    1,
    1,
    1
  ),
  (
    42,
    "test_game",
    1699804206419538067,
    1702569006419538067,
    1,
    3,
    2
  ),
  (
    6,
    "a_game",
    1573573806419538067,
    1573573806419538067,
    1,
    1,
    3
  );
