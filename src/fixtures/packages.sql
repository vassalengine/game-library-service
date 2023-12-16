INSERT INTO packages (package_id, project_id, name, created_at)
VALUES
  (1, 42, "a_package", "2023-12-09T15:56:29.180282477+00:00"),
  (2, 42, "b_package", "2022-11-06T15:56:29.180282477+00:00"),
  (3, 42, "c_package", "2023-11-06T15:56:29.180282477+00:00")
;

INSERT INTO package_versions (
  package_version_id,
  package_id,
  version,
  version_major,
  version_minor,
  version_patch,
  version_pre,
  version_build,
  url,
  filename,
  published_at
)
VALUES
  (1, 1, "1.2.3", 1, 2, 3, "", "", "https://example.com/a_package-1.2.3", "a_package-1.2.3", "2023-12-09T15:56:29.180282477+00:00"),
  (2, 1, "1.2.4", 1, 2, 4, "", "", "https://example.com/a_package-1.2.4", "a_package-1.2.4", "2023-12-10T15:56:29.180282477+00:00"),
  (3, 3, "0.1.0", 1, 2, 4, "", "", "https://example.com/c_package-0.1.0", "c_package-0.1.0", "2023-13-10T15:56:29.180282477+00:00");
