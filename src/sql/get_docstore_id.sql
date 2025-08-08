-- src/sql/get_docstore_id.sql
SELECT ds_id FROM docstores WHERE ds_name = ?1 AND p_id_fk = ?2;
