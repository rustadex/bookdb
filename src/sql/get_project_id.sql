-- src/sql/get_project_id.sql
SELECT p_id FROM projects WHERE p_name = ?1;
