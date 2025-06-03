#!/bin/bash -e

src="$1"
dst="$2"

cat << EOF | sqlite3 projects.db
UPDATE owners SET user_id = $dst WHERE user_id = $src;
UPDATE authors SET user_id = $dst WHERE user_id = $src;
UPDATE players SET user_id = $dst WHERE user_id = $src;
UPDATE packages SET created_by = $dst WHERE created_by = $src;
UPDATE releases SET published_by = $dst WHERE published_by = $src;
UPDATE files SET published_by = $dst WHERE published_by = $src;
UPDATE images SET published_by = $dst WHERE published_by = $src;
UPDATE image_revisions SET published_by = $dst WHERE published_by = $src;
UPDATE galleries SET published_by = $dst WHERE published_by = $src;
UPDATE galleries SET removed_by = $dst WHERE removed_by = $src;
UPDATE projects SET modified_by = $dst WHERE modified_by = $src;
UPDATE project_revisions SET modified_by = $dst WHERE modified_by = $src;
UPDATE flags SET user_id = $dst WHERE user_id = $src;
DELETE FROM users WHERE user_id = $src;
EOF
