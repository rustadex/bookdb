-- src/sql2/create_doc_store.sql
-- Create new document store (workspace.docstore)

INSERT INTO doc_stores (ds_name, pns_id_fk, workspace_name) 
VALUES (?1, ?2, ?3);
