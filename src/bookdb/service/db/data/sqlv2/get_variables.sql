-- src/sql2/get_variable.sql
-- Get variable value by key and context

SELECT v.var_value 
FROM vars v 
JOIN keyval_ns kvns ON v.kvns_id_fk = kvns.kvns_id 
JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
  AND kvns.workspace_name = ?2 
  AND kvns.kvns_name = ?3 
  AND v.var_key = ?4;
