-- src/sql/upsert_var.sql

INSERT INTO vars (var_key, var_value, var_updated, vs_id_fk)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(vs_id_fk, var_key) DO UPDATE SET
  var_value = excluded.var_value,
  var_updated = excluded.var_updated;
