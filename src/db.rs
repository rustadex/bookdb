use std::path::Path;
use rusqlite::{Connection, params, OptionalExtension};
use crate::error::{Result, BookdbError};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open_at(path: &Path) -> Result<Self> {
        let must_init = !path.exists();
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).ok(); }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        if must_init { db.bootstrap()?; } else { db.bootstrap()?; }
        Ok(db)
    }

    pub fn open_default() -> Result<Self> {
        Self::open_at(Path::new("bookdb.sqlite"))
    }

    fn bootstrap(&self) -> Result<()> {
        self.conn.execute_batch(crate::sql::V1_CREATE_TABLES)?;
        if !crate::sql::V2_CREATE_DOCS.is_empty() {
            self.conn.execute_batch(crate::sql::V2_CREATE_DOCS)?;
        }
        Ok(())
    }

    // --- Meta ---
    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let v: Option<String> = self.conn
            .query_row("SELECT value FROM meta WHERE key=?1", params![key], |r| r.get(0))
            .optional()?;
        Ok(v)
    }
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key,value) VALUES(?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value]
        )?;
        Ok(())
    }
}
