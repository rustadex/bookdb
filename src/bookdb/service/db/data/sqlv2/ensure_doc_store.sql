-- src/sql2/ensure_doc_store.sql
-- Ensure document store exists for workspace

INSERT OR IGNORE INTO doc_stores (ds_name, pns_id_fk, workspace_name) 
VALUES ('documents', ?1, ?2);
