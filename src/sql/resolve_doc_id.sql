INSERT INTO docs (doc_key, ds_id_fk, created_at, updated_at)
VALUES (?1, ?2, strftime('%s','now'), strftime('%s','now'))
ON CONFLICT(ds_id_fk, doc_key) DO UPDATE SET updated_at = excluded.updated_at
RETURNING doc_id;
