-- src/sql/list_diks.sql

SELECT dc_key FROM doc_chunks WHERE ds_id_fk = ?1 ORDER BY dc_key;
