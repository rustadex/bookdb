pub const V1_CREATE_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS projects(
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);
CREATE TABLE IF NOT EXISTS docstores(
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    UNIQUE(project_id, name)
);
CREATE TABLE IF NOT EXISTS varstores(
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    UNIQUE(project_id, name)
);
CREATE TABLE IF NOT EXISTS variables(
    id INTEGER PRIMARY KEY,
    vs_id_fk INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(vs_id_fk, key)
);
CREATE TABLE IF NOT EXISTS meta(
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS docs(
    id INTEGER PRIMARY KEY,
    ds_id_fk INTEGER NOT NULL,
    doc_key TEXT NOT NULL,
    UNIQUE(ds_id_fk, doc_key)
);
CREATE TABLE IF NOT EXISTS doc_segments(
    id INTEGER PRIMARY KEY,
    doc_id_fk INTEGER NOT NULL,
    path TEXT NOT NULL,
    mime TEXT NOT NULL,
    content BLOB NOT NULL,
    UNIQUE(doc_id_fk, path)
);
CREATE TABLE IF NOT EXISTS doc_chunks(
    id INTEGER PRIMARY KEY,
    ds_id_fk INTEGER NOT NULL,
    dc_key TEXT NOT NULL,
    dc_value TEXT NOT NULL,
    UNIQUE(ds_id_fk, dc_key)
);
"#;

pub const V2_CREATE_DOCS: &str = "";
