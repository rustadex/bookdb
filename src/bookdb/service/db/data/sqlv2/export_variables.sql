-- src/sql2/export_variables.sql
-- Export variables with context information

SELECT 
    v.var_key,
    v.var_value,
    pns.pns_name as project,
    kvns.workspace_name,
    kvns.kvns_name as keystore
FROM vars v 
JOIN keyval_ns kvns ON v.kvns_id_fk = kvns.kvns_id 
JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
WHERE (?1 IS NULL OR pns.pns_name LIKE '%' || ?1 || '%')
  AND (?2 IS NULL OR kvns.workspace_name LIKE '%' || ?2 || '%')
  AND (?3 IS NULL OR kvns.kvns_name LIKE '%' || ?3 || '%')
  AND (?4 IS NULL OR v.var_key LIKE '%' || ?4 || '%')
ORDER BY pns.pns_name, kvns.workspace_name, kvns.kvns_name, v.var_key;
