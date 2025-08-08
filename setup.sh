
# From inside the 'bookdb-rs' directory
mkdir -p src/commands src/sql
touch src/cli.rs
touch src/commands/mod.rs
touch src/commands/getv.rs
touch src/commands/setv.rs
touch src/commands/getd.rs
touch src/commands/setd.rs
touch src/db.rs
touch src/error.rs
touch src/models.rs
touch src/sql.rs
touch src/sql/V1__create_tables.sql
touch src/sql/get_var.sql

# From inside the 'bookdb-rs' directory
touch src/context.rs
touch src/sql/get_doc_chunk.sql
touch src/sql/upsert_doc_chunk.sql
touch src/sql/resolve_doc_context.sql
touch src/commands/import.rs
touch src/commands/export.rs

# From inside the 'bookdb-rs' directory
touch src/commands/ls.rs
touch src/sql/list_projects.sql
touch src/sql/list_docstores.sql
touch src/sql/list_varstores.sql
touch src/sql/list_keys.sql
touch src/sql/list_diks.sql

# From inside the 'bookdb-rs' directory
touch src/sql/get_project_id.sql
touch src/sql/get_docstore_id.sql
touch src/sql/get_varstore_id.sql

# bookdb-rs/
# ├── Cargo.toml
# └── src/
#     ├── cli.rs
#     ├── commands/
#     │   ├── mod.rs
#     │   ├── getd.rs
#     │   ├── getv.rs
#     │   ├── setd.rs
#     │   └── setv.rs
#     ├── db.rs
#     ├── error.rs
#     ├── main.rs
#     ├── models.rs
#     ├── sql.rs
#     └── sql/
#         ├── V1__create_tables.sql
#         └── get_var.sql


# RUST_LOG=info cargo run -- --context myapp.api.var.secrets --persist -- setv API_KEY=12345
