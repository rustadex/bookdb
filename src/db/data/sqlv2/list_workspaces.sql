-- src/sql2/list_workspaces.sql
-- List workspaces in a project

SELECT DISTINCT workspace_name 
FROM keyval_ns kvns 
JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
ORDER BY workspace_name;
