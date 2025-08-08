-- src/sql/resolve_docstore_id.sql

INSERT INTO docstores (ds_name, p_id_fk)
VALUES (?1, ?2)
ON CONFLICT(p_id_fk, ds_name) DO UPDATE SET ds_name = excluded.ds_name
RETURNING ds_id;
