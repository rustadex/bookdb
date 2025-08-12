-- src/sql2/resolve_doc_store_id.sql
-- Resolve document store (project.workspace.docstore) to ID

SELECT ds.ds_id 
FROM doc_stores ds 
JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
  AND ds.workspace_name = ?2 
  AND ds.ds_name = ?3;
