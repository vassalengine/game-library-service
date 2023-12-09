INSERT INTO projects (
  project_id,
  name,
  description,
  revision,
  created_at,
  modified_at,
  game_title,
  game_title_sort,
  game_publisher,
  game_year,
  readme_id
)
VALUES
  (
    42,
    "test_game",
    "Brian's Trademarked Game of Being a Test Case",
    1,
    "2023-11-12T15:50:06.419538067+00:00",
    "2023-11-12T15:50:06.419538067+00:00",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979",
    8
  ),
  (
    6,
    "a_game",
    "Another game",
    2,
    "2019-11-12T15:50:06.419538067+00:00",
    "2023-11-12T15:50:06.419538067+00:00",
    "Some Other Game",
    "Some Other Game",
    "XYZ",
    "1993",
    4
  );

INSERT INTO projects_revisions (
  project_id,
  name,
  description,
  revision,
  created_at,
  modified_at,
  game_title,
  game_title_sort,
  game_publisher,
  game_year,
  readme_id
)
VALUES
  (
    6,
    "a_game",
    "Another game",
    1,
    "2019-11-12T15:50:06.419538067+00:00",
    "2019-11-12T15:50:06.419538067+00:00",
    "Some Otter Game",
    "Some Otter Game",
    "Otters!",
    "1993",
    4
  );
