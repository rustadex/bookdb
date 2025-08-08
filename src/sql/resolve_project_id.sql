-- src/sql/resolve_project_id.sql

INSERT INTO projects (p_name)
VALUES (?1)
ON CONFLICT(p_name) DO UPDATE SET p_name = excluded.p_name
RETURNING p_id;
