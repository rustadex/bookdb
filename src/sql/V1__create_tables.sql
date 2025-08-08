-- src/sql/V1__create_tables.sql

-- Projects are the top-level containers within a base.
CREATE TABLE IF NOT EXISTS projects (
    p_id      INTEGER PRIMARY KEY,
    p_name    TEXT NOT NULL UNIQUE
);

-- Docstores belong to a project. They can contain varstores or doc_chunks.
CREATE TABLE IF NOT EXISTS docstores (
    ds_id     INTEGER PRIMARY KEY,
    ds_name   TEXT NOT NULL,
    p_id_fk   INTEGER NOT NULL,
    FOREIGN KEY (p_id_fk) REFERENCES projects(p_id) ON DELETE CASCADE,
    UNIQUE (p_id_fk, ds_name)
);

-- Varstores belong to a docstore. They contain variables.
CREATE TABLE IF NOT EXISTS varstores (
    vs_id     INTEGER PRIMARY KEY,
    vs_name   TEXT NOT NULL,
    ds_id_fk  INTEGER NOT NULL,
    FOREIGN KEY (ds_id_fk) REFERENCES docstores(ds_id) ON DELETE CASCADE,
    UNIQUE (ds_id_fk, vs_name)
);

-- Vars are the final key-value pairs within a varstore.
CREATE TABLE IF NOT EXISTS vars (
    var_id    INTEGER PRIMARY KEY,
    var_key   TEXT NOT NULL,
    var_value TEXT,
    var_updated INTEGER NOT NULL,
    vs_id_fk  INTEGER NOT NULL,
    FOREIGN KEY (vs_id_fk) REFERENCES varstores(vs_id) ON DELETE CASCADE,
    UNIQUE (vs_id_fk, var_key)
);

-- Doc_chunks are key-value pairs for documents, belonging to a docstore.
CREATE TABLE IF NOT EXISTS doc_chunks (
    dc_id     INTEGER PRIMARY KEY,
    dc_key    TEXT NOT NULL, -- The 'dik'
    dc_value  TEXT,
    dc_updated INTEGER NOT NULL,
    ds_id_fk  INTEGER NOT NULL,
    FOREIGN KEY (ds_id_fk) REFERENCES docstores(ds_id) ON DELETE CASCADE,
    UNIQUE (ds_id_fk, dc_key)
);
