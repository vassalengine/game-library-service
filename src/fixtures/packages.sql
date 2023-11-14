INSERT INTO packages (id, project_id, name)
VALUES
  (1, 42, "a_package"),
  (2, 42, "b_package")
;

INSERT INTO package_versions (id, package_id, version_major, version_minor, version_patch, url)
VALUES
  (1, 1, 1, 2, 3, "https://example.com/a_package-1.2.3"),
  (2, 1, 1, 2, 4, "https://example.com/a_package-1.2.4");
