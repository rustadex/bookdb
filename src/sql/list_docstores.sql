-- src/sql/list_docstores.sql

SELECT ds_name FROM docstores WHERE p_id_fk = ?1 ORDER BY ds_name;
