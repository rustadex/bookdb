use rusqlite::{Connection, params, OptionalExtension};
use crate::error::{Result, BookdbError};

pub struct Database { pub conn: Connection }

impl Database {
    pub fn open_default() -> Result<Self> {
        let conn = Connection::open("bookdb.sqlite")?;
        let db = Self{ conn };
        db.bootstrap()?;
        Ok(db)
    }

    fn bootstrap(&self) -> Result<()> {
        self.conn.execute_batch(crate::sql::V1_CREATE_TABLES)?;
        self.conn.execute_batch(crate::sql::V2_CREATE_DOCS)?;
        Ok(())
    }

    // --- Project/Docstore/Varstore ---
    pub fn get_or_create_project(&self, name: &str) -> Result<i64> {
        self.conn.execute("INSERT OR IGNORE INTO projects(name) VALUES (?1)", params![name])?;
        let id: i64 = self.conn.query_row("SELECT id FROM projects WHERE name=?1", params![name], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.query_to_vec("SELECT name FROM projects ORDER BY name", &[])
    }

    pub fn get_or_create_docstore(&self, project_id: i64, name: &str) -> Result<i64> {
        self.conn.execute("INSERT OR IGNORE INTO docstores(project_id,name) VALUES (?1,?2)", params![project_id, name])?;
        let id: i64 = self.conn.query_row("SELECT id FROM docstores WHERE project_id=?1 AND name=?2", params![project_id, name], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_docstores(&self) -> Result<Vec<String>> {
        self.query_to_vec("SELECT p.name||'.'||d.name FROM docstores d JOIN projects p ON p.id=d.project_id ORDER BY 1", &[])
    }

    pub fn get_or_create_varstore(&self, project_id: i64, name: &str) -> Result<i64> {
        self.conn.execute("INSERT OR IGNORE INTO varstores(project_id,name) VALUES (?1,?2)", params![project_id, name])?;
        let id: i64 = self.conn.query_row("SELECT id FROM varstores WHERE project_id=?1 AND name=?2", params![project_id, name], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_varstores(&self) -> Result<Vec<String>> {
        self.query_to_vec("SELECT p.name||'.'||v.name FROM varstores v JOIN projects p ON p.id=v.project_id ORDER BY 1", &[])
    }

    // --- Vars ---
    pub fn list_keys(&self, vs_id: i64) -> Result<Vec<String>> {
        self.query_to_vec("SELECT key FROM variables WHERE vs_id_fk=?1 ORDER BY key", &[&vs_id])
    }
    pub fn get_var(&self, key: &str, vs_id: i64) -> Result<Option<String>> {
        let v: Option<String> = self.conn.query_row("SELECT value FROM variables WHERE vs_id_fk=?1 AND key=?2",
            params![vs_id, key], |r| r.get(0)).optional()?;
        Ok(v)
    }
    pub fn set_var(&self, key: &str, value: &str, vs_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO variables(vs_id_fk,key,value) VALUES (?1,?2,?3)             ON CONFLICT(vs_id_fk,key) DO UPDATE SET value=excluded.value",
             params![vs_id, key, value])?;
        Ok(())
    }

    // --- Legacy chunks for migrate ---
    pub fn stream_doc_chunks(&self, ds_id: i64) -> Result<Vec<(String,String)>> {
        self.query_to_kv_vec("SELECT dc_key, dc_value FROM doc_chunks WHERE ds_id_fk=?1 ORDER BY dc_key", &[&ds_id])
    }

    // --- Docs/Segments V2 ---
    pub fn get_or_create_doc_id(&self, doc_key: &str, ds_id: i64) -> Result<i64> {
        self.conn.execute("INSERT OR IGNORE INTO docs(ds_id_fk, doc_key) VALUES (?1,?2)", params![ds_id, doc_key])?;
        let id: i64 = self.conn.query_row("SELECT id FROM docs WHERE ds_id_fk=?1 AND doc_key=?2", params![ds_id, doc_key], |r| r.get(0))?;
        Ok(id)
    }
    pub fn get_doc_id(&self, doc_key: &str, ds_id: i64) -> Result<i64> {
        let id: Option<i64> = self.conn.query_row("SELECT id FROM docs WHERE ds_id_fk=?1 AND doc_key=?2",
            params![ds_id, doc_key], |r| r.get(0)).optional()?;
        id.ok_or_else(|| BookdbError::KeyNotFound(doc_key.to_string()))
    }
    pub fn list_docs_v2(&self, ds_id: i64) -> Result<Vec<String>> {
        self.query_to_vec("SELECT doc_key FROM docs WHERE ds_id_fk=?1 ORDER BY doc_key", &[&ds_id])
    }
    pub fn list_segments(&self, doc_key: &str, ds_id: i64) -> Result<Vec<String>> {
        let doc_id = self.get_doc_id(doc_key, ds_id)?;
        self.query_to_vec("SELECT path FROM doc_segments WHERE doc_id_fk=?1 ORDER BY path", &[&doc_id])
    }
    pub fn get_doc_segment(&self, doc_key: &str, path: &str, ds_id: i64) -> Result<Option<(Vec<u8>, String)>> {
        let doc_id = self.get_doc_id(doc_key, ds_id)?;
        let row: Option<(Vec<u8>, String)> = self.conn.query_row(
            "SELECT content, mime FROM doc_segments WHERE doc_id_fk=?1 AND path=?2",
            params![doc_id, path], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
        Ok(row)
    }
    pub fn set_doc_segment(&self, doc_key: &str, path: &str, mime: &str, content: &[u8], ds_id: i64) -> Result<()> {
        let doc_id = self.get_or_create_doc_id(doc_key, ds_id)?;
        self.conn.execute(
            "INSERT INTO doc_segments(doc_id_fk,path,mime,content) VALUES (?1,?2,?3,?4)             ON CONFLICT(doc_id_fk,path) DO UPDATE SET mime=excluded.mime, content=excluded.content",
             params![doc_id, path, mime, content])?;
        Ok(())
    }

    // --- Helpers ---
    pub fn query_to_vec(&self, sql: &str, paramsx: &[&dyn rusqlite::ToSql]) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(paramsx, |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for v in rows { out.push(v?); }
        Ok(out)
    }
    pub fn query_to_kv_vec(&self, sql: &str, paramsx: &[&dyn rusqlite::ToSql]) -> Result<Vec<(String,String)>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(paramsx, |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = Vec::new();
        for v in rows { out.push(v?); }
        Ok(out)
    }
}
