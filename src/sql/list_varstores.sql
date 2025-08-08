-- src/sql/list_varstores.sql

SELECT vs_name FROM varstores WHERE ds_id_fk = ?1 ORDER BY vs_name;
