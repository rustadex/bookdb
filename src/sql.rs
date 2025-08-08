// src/sql.rs

/// Contains all SQL queries as compile-time constants.
/// This prevents SQL typos and keeps logic separate from query text.

// --- Schema ---
pub const V1_CREATE_TABLES: &str = include_str!("sql/V1__create_tables.sql");

// --- Get-Only Resolution ---
pub const GET_PROJECT_ID: &str = include_str!("sql/get_project_id.sql");
pub const GET_DOCSTORE_ID: &str = include_str!("sql/get_docstore_id.sql");
pub const GET_VARSTORE_ID: &str = include_str!("sql/get_varstore_id.sql");

// --- Get-or-Create Resolution ---
pub const RESOLVE_PROJECT_ID: &str = include_str!("sql/resolve_project_id.sql");
pub const RESOLVE_DOCSTORE_ID: &str = include_str!("sql/resolve_docstore_id.sql");
pub const RESOLVE_VARSTORE_ID: &str = include_str!("sql/resolve_varstore_id.sql");

// --- List Queries ---
pub const LIST_PROJECTS: &str = include_str!("sql/list_projects.sql");
pub const LIST_DOCSTORES: &str = include_str!("sql/list_docstores.sql");
pub const LIST_VARSTORES: &str = include_str!("sql/list_varstores.sql");
pub const LIST_KEYS: &str = include_str!("sql/list_keys.sql");
pub const LIST_DIKS: &str = include_str!("sql/list_diks.sql");

// --- Variables ---
pub const GET_VAR: &str = include_str!("sql/get_var.sql");
pub const UPSERT_VAR: &str = include_str!("sql/upsert_var.sql");

// --- Documents ---
pub const GET_DOC_CHUNK: &str = include_str!("sql/get_doc_chunk.sql");
pub const UPSERT_DOC_CHUNK: &str = include_str!("sql/upsert_doc_chunk.sql");


// --- V2 Docs/Segments ---
pub const V2_CREATE_DOCS: &str = include_str!("sql/V2__create_docs.sql");
pub const GET_DOC_ID: &str = include_str!("sql/get_doc_id.sql");
pub const RESOLVE_DOC_ID: &str = include_str!("sql/resolve_doc_id.sql");
pub const GET_DOC_SEGMENT: &str = include_str!("sql/get_doc_segment.sql");
pub const UPSERT_DOC_SEGMENT: &str = include_str!("sql/upsert_doc_segment.sql");
pub const LIST_DOCS_V2: &str = include_str!("sql/list_docs_v2.sql");
pub const LIST_SEGMENTS: &str = include_str!("sql/list_segments.sql");
