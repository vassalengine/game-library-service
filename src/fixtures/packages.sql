INSERT INTO packages (package_id, project_id, name)
VALUES
  (1, 42, "a_package"),
  (2, 42, "b_package")
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
  filename
)
VALUES
  (1, 1, "1.2.3", 1, 2, 3, "", "", "https://example.com/a_package-1.2.3", "a_package-1.2.3"),
  (2, 1, "1.2.4", 1, 2, 4, "", "", "https://example.com/a_package-1.2.4", "a_package-1.2.4");
