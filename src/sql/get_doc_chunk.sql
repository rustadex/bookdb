-- src/sql/get_doc_chunk.sql

-- Retrieves a single document chunk's value based on its key (dik) and parent docstore ID.
SELECT dc_value
FROM doc_chunks
WHERE dc_key = ?1 AND ds_id_fk = ?2;
