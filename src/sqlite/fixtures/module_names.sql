UPDATE files
SET module_name = "Test Module"
WHERE file_id = 1;

/* the aggregate normally maintained by update_module_names; updating it
   also updates the projects_fts index via the projects_au trigger */
UPDATE projects
SET module_names = "Test Module"
WHERE project_id = 42;
