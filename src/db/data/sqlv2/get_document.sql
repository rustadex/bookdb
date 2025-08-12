-- src/sql2/get_document.sql
-- Get document content by key and context

SELECT d.doc_content 
FROM docs d 
JOIN doc_stores ds ON d.ds_id_fk = ds.ds_id 
JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
WHERE pns.pns_name = ?1 
  AND ds.workspace_name = ?2 
  AND d.doc_key = ?3;
