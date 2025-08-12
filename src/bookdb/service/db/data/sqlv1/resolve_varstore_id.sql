-- src/sql/resolve_varstore_id.sql

INSERT INTO varstores (vs_name, ds_id_fk)
VALUES (?1, ?2)
ON CONFLICT(ds_id_fk, vs_name) DO UPDATE SET vs_name = excluded.vs_name
RETURNING vs_id;
