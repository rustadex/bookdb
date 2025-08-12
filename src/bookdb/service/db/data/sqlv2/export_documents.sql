-- src/sql2/export_documents.sql
-- Export documents with context information

SELECT 
    d.doc_key,
    d.doc_content,
    pns.pns_name as project,
    ds.workspace_name
FROM docs d 
JOIN doc_stores ds ON d.ds_id_fk = ds.ds_id 
JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
WHERE (?1 IS NULL OR pns.pns_name LIKE '%' || ?1 || '%')
  AND (?2 IS NULL OR ds.workspace_name LIKE '%' || ?2 || '%')
  AND (?3 IS NULL OR d.doc_key LIKE '%' || ?3 || '%')
ORDER BY pns.pns_name, ds.workspace_name, d.doc_key;
