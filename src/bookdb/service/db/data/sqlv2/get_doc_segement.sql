-- src/sql2/get_doc_segment.sql
-- Get document segment content

SELECT ds.content 
FROM doc_segments ds 
JOIN docs d ON ds.doc_id_fk = d.doc_id 
WHERE d.doc_key = ?1 
  AND ds.path = ?2;
