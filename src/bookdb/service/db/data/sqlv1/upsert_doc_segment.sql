INSERT INTO doc_segments (path, mime, content, updated_at, doc_id_fk)
VALUES (?1, ?2, ?3, strftime('%s','now'), ?4)
ON CONFLICT(doc_id_fk, path) DO UPDATE SET
  content = excluded.content,
  mime = excluded.mime,
  updated_at = excluded.updated_at;
