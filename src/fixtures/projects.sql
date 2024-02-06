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
  (
    42,
    "test_game",
    1699804206419538067,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979",
    "",
    NULL,
    1702569006419538067,
    1,
    3
  ),
  (
    6,
    "a_game",
    1573573806419538067,
    "Another game",
    "Some Other Game",
    "Some Other Game",
    "XYZ",
    "1993",
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
  readme,
  image
)
VALUES
  (
    1,
    42,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1978",
    "",
    NULL
  ),
  (
    2,
    42,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979",
    "",
    NULL
  );

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
  );
