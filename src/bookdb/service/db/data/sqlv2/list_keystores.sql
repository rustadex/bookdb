-- src/sql2/list_keystores.sql
-- List keystores in a workspace within a project

SELECT kvns.kvns_name 
FROM keyval_ns kvns 
JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
  AND kvns.workspace_name = ?2 
ORDER BY kvns.kvns_name;
