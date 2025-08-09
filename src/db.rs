use std::path::{Path, PathBuf};
use rusqlite::{Connection, params, OptionalExtension};
use crate::error::{Result, BookdbError};

pub struct Database {
    pub _path: PathBuf,
    pub conn: Connection,
}

impl Database {
    pub fn open_at(path: &Path) -> Result<Self> {
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
        let conn = Connection::open(path)?;
        Ok(Self { _path: path.to_path_buf(), conn })
    }

    pub fn is_installed(&self) -> Result<bool> {
        let exists: bool = self.conn
            .query_row("SELECT value FROM meta WHERE key='installed'", [], |r| {
                let v: String = r.get(0)?; Ok(v == "1")
            })
            .optional()? 
            .unwrap_or(false);
        Ok(exists)
    }

    pub fn bootstrap_schema(&self) -> Result<()> {
        self.conn.execute_batch(crate::sql::V1_CREATE_TABLES)?;
        Ok(())
    }

    pub fn mark_installed(&self) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key,value) VALUES('installed','1')
             ON CONFLICT(key) DO UPDATE SET value='1'",
            []
        )?;
        Ok(())
    }

    pub fn seed_home_invincibles(&self) -> Result<()> {
        let proj_id = self.get_or_create_project_id("home")?;
        let ds_id   = self.get_or_create_docstore_id(proj_id, "GLOBAL")?;
        let _vs_id  = self.get_or_create_varstore_id(proj_id, "MAIN")?;
        let _doc_id = self.get_or_create_doc_id("MAIN", ds_id)?;
        Ok(())
    }

    pub fn get_or_create_project_id(&self, name: &str) -> Result<i64> {
        Self::ensure_not_reserved(name, "project")?;
        self.conn.execute("INSERT OR IGNORE INTO projects(name) VALUES(?1)", params![name])?;
        let id: i64 = self.conn.query_row("SELECT id FROM projects WHERE name=?1", params![name], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut st = self.conn.prepare("SELECT name FROM projects ORDER BY name")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        for v in rows { out.push(v?); }
        Ok(out)
    }

    pub fn get_or_create_docstore_id(&self, project_id: i64, name: &str) -> Result<i64> {
        Self::ensure_not_reserved(name, "docstore")?;
        self.conn.execute("INSERT OR IGNORE INTO docstores(project_id,name) VALUES(?1,?2)", params![project_id, name])?;
        let id: i64 = self.conn.query_row("SELECT id FROM docstores WHERE project_id=?1 AND name=?2", params![project_id, name], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_docstores(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut st = self.conn.prepare("SELECT p.name||'.'||d.name FROM docstores d JOIN projects p ON p.id=d.project_id ORDER BY 1")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        for v in rows { out.push(v?); }
        Ok(out)
    }

    pub fn get_or_create_varstore_id(&self, project_id: i64, name: &str) -> Result<i64> {
        Self::ensure_not_reserved(name, "varstore")?;
        self.conn.execute("INSERT OR IGNORE INTO varstores(project_id,name) VALUES(?1,?2)", params![project_id, name])?;
        let id: i64 = self.conn.query_row("SELECT id FROM varstores WHERE project_id=?1 AND name=?2", params![project_id, name], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_varstores(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut st = self.conn.prepare("SELECT p.name||'.'||v.name FROM varstores v JOIN projects p ON p.id=v.project_id ORDER BY 1")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        for v in rows { out.push(v?); }
        Ok(out)
    }

    pub fn get_or_create_doc_id(&self, doc_key: &str, ds_id: i64) -> Result<i64> {
        Self::ensure_not_reserved(doc_key, "doc key")?;
        self.conn.execute("INSERT OR IGNORE INTO docs(ds_id_fk,doc_key) VALUES(?1,?2)", params![ds_id, doc_key])?;
        let id: i64 = self.conn.query_row("SELECT id FROM docs WHERE ds_id_fk=?1 AND doc_key=?2", params![ds_id, doc_key], |r| r.get(0))?;
        Ok(id)
    }
    pub fn list_docs_v2(&self, ds_id: i64) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut st = self.conn.prepare("SELECT doc_key FROM docs WHERE ds_id_fk=?1 ORDER BY doc_key")?;
        let rows = st.query_map(params![ds_id], |r| r.get::<_, String>(0))?;
        for v in rows { out.push(v?); }
        Ok(out)
    }

    // VARS
    pub fn get_var(&self, key: &str, vs_id: i64) -> Result<Option<String>> {
        let v: Option<String> = self.conn
            .query_row("SELECT value FROM variables WHERE vs_id_fk=?1 AND key=?2", params![vs_id, key], |r| r.get(0))
            .optional()?;
        Ok(v)
    }
    pub fn set_var(&self, key: &str, value: &str, vs_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO variables(vs_id_fk,key,value) VALUES(?1,?2,?3)
             ON CONFLICT(vs_id_fk,key) DO UPDATE SET value=excluded.value",
            params![vs_id, key, value]
        )?;
        Ok(())
    }
    pub fn list_keys(&self, vs_id: i64) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut st = self.conn.prepare("SELECT key FROM variables WHERE vs_id_fk=?1 ORDER BY key")?;
        let rows = st.query_map(params![vs_id], |r| r.get::<_, String>(0))?;
        for v in rows { out.push(v?); }
        Ok(out)
    }

    // DOCS
    pub fn get_doc_segment(&self, doc_key: &str, path: &str, ds_id: i64) -> Result<Option<(Vec<u8>, String)>> {
        let doc_id: Option<i64> = self.conn
            .query_row("SELECT id FROM docs WHERE ds_id_fk=?1 AND doc_key=?2", params![ds_id, doc_key], |r| r.get(0))
            .optional()?;
        let Some(doc_id) = doc_id else { return Ok(None); };
        let row: Option<(Vec<u8>, String)> = self.conn
            .query_row("SELECT content,mime FROM doc_segments WHERE doc_id_fk=?1 AND path=?2",
                       params![doc_id, path], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()?;
        Ok(row)
    }
    pub fn set_doc_segment(&self, doc_key: &str, path: &str, mime: &str, content: &[u8], ds_id: i64) -> Result<()> {
        let doc_id = self.get_or_create_doc_id(doc_key, ds_id)?;
        self.conn.execute(
            "INSERT INTO doc_segments(doc_id_fk,path,mime,content) VALUES(?1,?2,?3,?4)
             ON CONFLICT(doc_id_fk,path) DO UPDATE SET mime=excluded.mime, content=excluded.content",
            params![doc_id, path, mime, content]
        )?;
        Ok(())
    }
    pub fn list_segments(&self, doc_key: &str, ds_id: i64) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let doc_id: Option<i64> = self.conn
            .query_row("SELECT id FROM docs WHERE ds_id_fk=?1 AND doc_key=?2", params![ds_id, doc_key], |r| r.get(0))
            .optional()?;
        if let Some(doc_id) = doc_id {
            let mut st = self.conn.prepare("SELECT path FROM doc_segments WHERE doc_id_fk=?1 ORDER BY path")?;
            let rows = st.query_map(params![doc_id], |r| r.get::<_, String>(0))?;
            for v in rows { out.push(v?); }
        }
        Ok(out)
    }

    pub fn stream_doc_chunks(&self, _ds_id: i64) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }

    pub fn ensure_not_reserved(name: &str, kind: &str) -> Result<()> {
        let up = name.to_ascii_uppercase();
        if up == "VAR" || up == "DOC" {
            return Err(BookdbError::ContextParse(format!("{} name '{}' is reserved", kind, name)));
        }
        Ok(())
    }
}
