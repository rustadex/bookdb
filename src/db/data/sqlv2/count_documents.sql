-- src/sql2/count_documents.sql
-- Count documents in a workspace

SELECT COUNT(*) 
FROM docs d 
JOIN doc_stores ds ON d.ds_id_fk = ds.ds_id 
JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
  AND ds.workspace_name = ?2;
