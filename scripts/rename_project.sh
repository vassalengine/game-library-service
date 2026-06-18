#!/bin/bash -e

proj_id="$1"
new_name="$2"

norm=$(python3 scripts/proj_norm.py "$2")
slug=$(python3 scripts/proj_slug.py "$2")

cat << EOF | sqlite3 projects.db
PRAGMA foreign_keys = ON;
BEGIN TRANSACTION;
UPDATE projects SET name = '$new_name', normalized_name = '$norm', slug = '$slug' WHERE project_id = $proj_id;
UPDATE projects_data SET name = '$new_name', slug = '$slug' WHERE project_id = $proj_id;
COMMIT TRANSACTION;
EOF
