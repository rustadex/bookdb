-- src/sql2/get_doc_store_id.sql
-- Get document store ID for workspace

SELECT ds.ds_id 
FROM doc_stores ds 
JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
  AND ds.workspace_name = ?2;
