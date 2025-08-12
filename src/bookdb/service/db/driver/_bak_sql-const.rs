// src/sql.rs - SQL query loader using sql2 files

// Schema creation files
pub const V1_CREATE_TABLES: &str = include_str!("sql2/V1__create_tables.sql");
pub const V2_CREATE_DOCS: &str = include_str!("sql2/V2__create_docs.sql");

// Context resolution queries
pub const RESOLVE_PROJECT_ID: &str = include_str!("sql2/resolve_project_id.sql");
pub const RESOLVE_KEYVAL_NS_ID: &str = include_str!("sql2/resolve_keyval_ns_id.sql");
pub const RESOLVE_DOC_STORE_ID: &str = include_str!("sql2/resolve_doc_store_id.sql");

// Creation queries
pub const CREATE_PROJECT: &str = include_str!("sql2/create_project.sql");
pub const CREATE_KEYVAL_NS: &str = include_str!("sql2/create_keyval_ns.sql");
pub const CREATE_DOC_STORE: &str = include_str!("sql2/create_doc_store.sql");

// Variable operations
pub const GET_VARIABLE: &str = include_str!("sql2/get_variable.sql");
pub const SET_VARIABLE: &str = include_str!("sql2/set_variable.sql");
pub const DELETE_VARIABLE: &str = include_str!("sql2/delete_variable.sql");

// Document operations
pub const GET_DOCUMENT: &str = include_str!("sql2/get_document.sql");
pub const SET_DOCUMENT: &str = include_str!("sql2/set_document.sql");
pub const DELETE_DOCUMENT: &str = include_str!("sql2/delete_document.sql");

// Document segment operations
pub const GET_DOC_SEGMENT: &str = include_str!("sql2/get_doc_segment.sql");
pub const SET_DOC_SEGMENT: &str = include_str!("sql2/set_doc_segment.sql");

// Listing operations
pub const LIST_PROJECTS: &str = include_str!("sql2/list_projects.sql");
pub const LIST_WORKSPACES: &str = include_str!("sql2/list_workspaces.sql");
pub const LIST_KEYSTORES: &str = include_str!("sql2/list_keystores.sql");
pub const LIST_VARIABLES: &str = include_str!("sql2/list_variables.sql");
pub const LIST_DOCUMENTS: &str = include_str!("sql2/list_documents.sql");

// Counting operations
pub const COUNT_VARIABLES: &str = include_str!("sql2/count_variables.sql");
pub const COUNT_DOCUMENTS: &str = include_str!("sql2/count_documents.sql");

// Document store management
pub const ENSURE_DOC_STORE: &str = include_str!("sql2/ensure_doc_store.sql");
pub const GET_DOC_STORE_ID: &str = include_str!("sql2/get_doc_store_id.sql");

// Export operations
pub const EXPORT_VARIABLES: &str = include_str!("sql2/export_variables.sql");
pub const EXPORT_DOCUMENTS: &str = include_str!("sql2/export_documents.sql");
