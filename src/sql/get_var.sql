-- src/sql/get_var.sql

-- Retrieves a single variable's value based on its key and parent varstore ID.
SELECT var_value
FROM vars
WHERE var_key = ?1 AND vs_id_fk = ?2;
