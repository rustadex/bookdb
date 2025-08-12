-- src/sql2/V2__create_docs.sql
-- Document segments for rich document support

CREATE TABLE IF NOT EXISTS doc_segments (
    seg_id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    mime TEXT NOT NULL DEFAULT 'text/plain',
    content BLOB NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    doc_id_fk INTEGER NOT NULL,
    FOREIGN KEY (doc_id_fk) REFERENCES docs(doc_id) ON DELETE CASCADE,
    UNIQUE (doc_id_fk, path)
);
