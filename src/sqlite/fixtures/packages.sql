INSERT INTO packages_history (
  package_id,
  project_id,
  created_at,
  created_by,
  deleted_at,
  deleted_by
)
VALUES
  (1, 42, 1702137389180282477, 1, NULL, NULL),
  (2, 42, 1667750189180282477, 1, NULL, NULL),
  (3, 42, 1699286189180282477, 1, NULL, NULL),
  (4, 42, 1699286189180282477, 1, 1699286189180282480, 1)
;

INSERT INTO packages_revisions (
  package_id,
  name,
  slug,
  sort_key,
  modified_at,
  modified_by
)
VALUES
  (1, "a_package", "a_package", 0, 1702137389180282477, 1),
  (2, "b_package", "b_package", 1, 1667750189180282477, 1),
  (3, "c_package", "c_package", 2, 1699286189180282477, 1),
  (4, "d_package", "d_package", 3, 1699286189180282477, 1)
;

INSERT INTO packages (
  package_id,
  project_id,
  name,
  slug,
  sort_key,
  created_at,
  created_by
)
VALUES
  (1, 42, "a_package", "a_package", 0, 1702137389180282477, 1),
  (2, 42, "b_package", "b_package", 1, 1667750189180282477, 1),
  (3, 42, "c_package", "c_package", 2, 1699286189180282477, 1)
;

INSERT INTO releases_history (
  release_id,
  package_id,
  version,
  version_major,
  version_minor,
  version_patch,
  version_pre,
  version_build,
  published_at,
  published_by,
  deleted_at,
  deleted_by
)
VALUES
  (
    1,
    1,
    "1.2.3",
    1,
    2,
    3,
    "",
    "",
    1702137389180282477,
    1,
    NULL,
    NULL
  ),
  (
    2,
    1,
    "1.2.4",
    1,
    2,
    4,
    "",
    "",
    1702223789180282477,
    2,
    NULL,
    NULL
  ),
  (
    3,
    3,
    "0.1.0",
    1,
    2,
    4,
    "",
    "",
    1702655789180282477,
    3,
    NULL,
    NULL
  ),
  (
    4,
    1,
    "1.2.5",
    1,
    2,
    5,
    "",
    "",
    1702223789180282477,
    2,
    NULL,
    NULL
  );

INSERT INTO releases (
  release_id,
  package_id,
  version,
  version_major,
  version_minor,
  version_patch,
  version_pre,
  version_build,
  published_at,
  published_by
)
VALUES
  (
    1,
    1,
    "1.2.3",
    1,
    2,
    3,
    "",
    "",
    1702137389180282477,
    1
  ),
  (
    2,
    1,
    "1.2.4",
    1,
    2,
    4,
    "",
    "",
    1702223789180282477,
    2
  ),
  (
    3,
    3,
    "0.1.0",
    1,
    2,
    4,
    "",
    "",
    1702655789180282477,
    3
  ),
  (
    4,
    1,
    "1.2.5",
    1,
    2,
    5,
    "",
    "",
    1702223789180282477,
    2
  );

INSERT INTO files (
  file_id,
  release_id,
  url,
  filename,
  size,
  sha256,
  requires,
  published_at,
  published_by
)
VALUES
  (
    1,
    1,
    "https://example.com/a_package-1.2.3",
    "a_package-1.2.3",
    1234,
    "c0e0fa7373a12b45a91e4f4d4e2e186442fc6ee9b346caa2fdc1c09026a2144a",
    ">= 3.2.17",
    1702137389180282477,
    1
  ),
  (
    2,
    2,
    "https://example.com/a_package-1.2.4",
    "a_package-1.2.4",
    5678,
    "79fdd8fe3128f818e446e919cce5dcfb81815f8f4341c53f4d6b58ded48cebf2",
    ">= 3.7.12",
    1702223789180282477,
    2
  ),
  (
    3,
    3,
    "https://example.com/c_package-0.1.0",
    "c_package-0.1.0",
     123456,
    "a8f515e9e2de99919d1a987733296aaa951a4ba2aa0f7014c510bdbd60dc0efd",
    NULL,
    1702655789180282477,
    3
  );
