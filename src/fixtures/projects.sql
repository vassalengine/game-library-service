INSERT INTO projects (
  project_id,
  name,
  created_at
)
VALUES
  (42, "test_game", "2023-11-12T15:50:06.419538067+00:00"),
  (6, "a_game", "2019-11-12T15:50:06.419538067+00:00");

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
  (
    1,
    42,
    "Brian's Trademarked Game of Being a Test Case",
    "A Game of Tests",
    "Game of Tests, A",
    "Test Game Company",
    "1979"
  ),
  (
    2,
    42,
    "Another game",
    "Some Other Game",
    "Some Other Game",
    "XYZ",
    "1993"
  ),
  (
    3,
    42,
    "Another game",
    "Some Otter Game",
    "Some Otter Game",
    "Otters!",
    "1993"
  );

INSERT INTO project_revisions (
  project_id,
  revision,
  project_data_id,
  readme_id,
  modified_at
)
VALUES
  (42, 1, 1, 8, "2023-11-12T15:50:06.419538067+00:00"),
  (42, 2, 1, 8, "2023-12-12T15:50:06.419538067+00:00"),
  (42, 3, 1, 8, "2023-12-14T15:50:06.419538067+00:00");
