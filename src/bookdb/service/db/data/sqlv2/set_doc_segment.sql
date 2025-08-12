-- src/sql2/set_doc_segment.sql
-- Set document segment content (upsert)

INSERT INTO doc_segments (path, mime, content, doc_id_fk, updated_at) 
VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))
ON CONFLICT (doc_id_fk, path) 
DO UPDATE SET 
    mime = excluded.mime,
    content = excluded.content,
    updated_at = excluded.updated_at;
