CREATE TABLE IF NOT EXISTS docs (
    doc_id     INTEGER PRIMARY KEY,
    doc_key    TEXT NOT NULL,
    ds_id_fk   INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY (ds_id_fk) REFERENCES docstores(ds_id) ON DELETE CASCADE,
    UNIQUE (ds_id_fk, doc_key)
);
CREATE TABLE IF NOT EXISTS doc_segments (
    seg_id     INTEGER PRIMARY KEY,
    path       TEXT NOT NULL,
    mime       TEXT NOT NULL DEFAULT 'text/plain',
    content    BLOB NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    doc_id_fk  INTEGER NOT NULL,
    FOREIGN KEY (doc_id_fk) REFERENCES docs(doc_id) ON DELETE CASCADE,
    UNIQUE (doc_id_fk, path)
);
