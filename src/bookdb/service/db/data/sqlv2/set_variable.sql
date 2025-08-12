-- src/sql2/set_variable.sql
-- Set variable value (upsert)

INSERT INTO vars (var_key, var_value, kvns_id_fk, var_updated) 
VALUES (?1, ?2, ?3, strftime('%s','now'))
ON CONFLICT (var_key, kvns_id_fk) 
DO UPDATE SET 
    var_value = excluded.var_value,
    var_updated = excluded.var_updated;
