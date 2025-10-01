INSERT INTO flags (
  flag_id,
  user_id,
  project_id,
  flagged_at,
  closed_at,
  closed_by,
  flag,
  message
)
VALUES
  (1, 1, 42, 1699804206419538067, NULL, NULL, 1, NULL),
  (2, 3, 42, 1699804206419538067, 1699804206419539067, 1, 0, NULL);
