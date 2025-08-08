-- src/sql/upsert_doc_chunk.sql

INSERT INTO doc_chunks (dc_key, dc_value, dc_updated, ds_id_fk)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(ds_id_fk, dc_key) DO UPDATE SET
  dc_value = excluded.dc_value,
  dc_updated = excluded.dc_updated;
