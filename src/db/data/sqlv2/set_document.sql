-- src/sql2/set_document.sql
-- Set document content (upsert)

INSERT INTO docs (doc_key, doc_content, ds_id_fk, doc_updated) 
VALUES (?1, ?2, ?3, strftime('%s','now'))
ON CONFLICT (doc_key, ds_id_fk) 
DO UPDATE SET 
    doc_content = excluded.doc_content,
    doc_updated = excluded.doc_updated;
