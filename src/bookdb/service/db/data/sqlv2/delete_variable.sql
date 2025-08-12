-- src/sql2/delete_variable.sql
-- Delete variable by key and context

DELETE FROM vars 
WHERE var_id IN (
    SELECT v.var_id 
    FROM vars v 
    JOIN keyval_ns kvns ON v.kvns_id_fk = kvns.kvns_id 
    JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
    WHERE pns.pns_name = ?1 
      AND kvns.workspace_name = ?2 
      AND kvns.kvns_name = ?3 
      AND v.var_key = ?4
);
