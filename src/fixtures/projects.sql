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
  revision
)
VALUES
  (
    42,
    "test_game",
    "2023-11-12T15:50:06.419538067+00:00",
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979",
    "",
    NULL,
    "2023-12-14T15:50:06.419538067+00:00",
    3
  ),
  (
    6,
    "a_game",
    "2019-11-12T15:50:06.419538067+00:00",
    "Another game",
    "Some Other Game",
    "Some Other Game",
    "XYZ",
    "1993",
    "",
    NULL,
    "2019-11-12T15:50:06.419538067+00:00",
    1
  );

INSERT INTO projects_arch (
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
  revision
)
VALUES
  (
    42,
    "test_game",
    "2023-11-12T15:50:06.419538067+00:00",
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979",
    "",
    NULL,
    "2023-11-12T15:50:06.419538067+00:00",
    1
  ),
  (
    42,
    "test_game",
    "2023-11-12T15:50:06.419538067+00:00",
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979",
    "",
    NULL,
    "2023-12-12T15:50:06.419538067+00:00",
    2
  );
