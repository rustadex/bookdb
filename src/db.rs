// src/db.rs

use crate::error::{BookdbError, Result};
use crate::models::{Context, Namespace};
use crate::sql;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute("PRAGMA foreign_keys = ON;", [])?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(sql::V1_CREATE_TABLES)?;
        Ok(())
    }

    pub fn backup_db(&self, backup_path: &Path) -> Result<()> {
        self.conn
            .backup(rusqlite::DatabaseName::Main, backup_path, None)
            .map_err(BookdbError::from)
    }
    
    // ... all other methods from the last correct version of this file remain here ...
    pub fn get_var_context_ids(&self, context: &Context) -> Result<(i64, i64, i64)> {
        let varstore_name = match &context.active_namespace {
            Namespace::Variables { varstore_name } => varstore_name,
            Namespace::Document => return Err(BookdbError::ContextParse("Expected variable context".into())),
        };

        let (p_id, ds_id) = self.get_doc_context_ids(context)?;
        
        let vs_id: i64 = self.conn.query_row(
            sql::GET_VARSTORE_ID,
            params![varstore_name, ds_id],
            |row| row.get(0),
        ).optional()?.ok_or_else(|| BookdbError::NamespaceNotFound(format!("varstore '{}'", varstore_name)))?;

        Ok((p_id, ds_id, vs_id))
    }

    pub fn get_doc_context_ids(&self, context: &Context) -> Result<(i64, i64)> {
        let p_id: i64 = self.conn.query_row(
            sql::GET_PROJECT_ID,
            params![&context.project_name],
            |row| row.get(0),
        ).optional()?.ok_or_else(|| BookdbError::NamespaceNotFound(format!("project '{}'", context.project_name)))?;

        let ds_id: i64 = self.conn.query_row(
            sql::GET_DOCSTORE_ID,
            params![&context.docstore_name, p_id],
            |row| row.get(0),
        ).optional()?.ok_or_else(|| BookdbError::NamespaceNotFound(format!("docstore '{}'", context.docstore_name)))?;

        Ok((p_id, ds_id))
    }

    pub fn resolve_var_context_or_create(&self, context: &Context) -> Result<(i64, i64, i64)> {
        let varstore_name = match &context.active_namespace {
            Namespace::Variables { varstore_name } => varstore_name,
            Namespace::Document => return Err(BookdbError::ContextParse("Expected variable context".into())),
        };

        let (p_id, ds_id) = self.resolve_doc_context_or_create(context)?;

        let vs_id: i64 = self.conn.query_row(
            sql::RESOLVE_VARSTORE_ID,
            params![varstore_name, ds_id],
            |row| row.get(0),
        )?;

        Ok((p_id, ds_id, vs_id))
    }

    pub fn resolve_doc_context_or_create(&self, context: &Context) -> Result<(i64, i64)> {
        let p_id: i64 = self.conn.query_row(
            sql::RESOLVE_PROJECT_ID,
            params![&context.project_name],
            |row| row.get(0),
        )?;

        let ds_id: i64 = self.conn.query_row(
            sql::RESOLVE_DOCSTORE_ID,
            params![&context.docstore_name, p_id],
            |row| row.get(0),
        )?;

        Ok((p_id, ds_id))
    }
    
    pub fn get_var(&self, key: &str, vs_id: i64) -> Result<Option<String>> {
        self.conn.query_row(sql::GET_VAR, params![key, vs_id], |row| row.get(0)).optional().map_err(BookdbError::from)
    }

    pub fn set_var(&self, key: &str, value: &str, vs_id: i64) -> Result<()> {
        let now = Utc::now().timestamp();
        self.conn.execute(sql::UPSERT_VAR, params![key, value, now, vs_id])?;
        Ok(())
    }

    pub fn get_doc_chunk(&self, dik: &str, ds_id: i64) -> Result<Option<String>> {
        self.conn.query_row(sql::GET_DOC_CHUNK, params![dik, ds_id], |row| row.get(0)).optional().map_err(BookdbError::from)
    }

    pub fn set_doc_chunk(&self, dik: &str, value: &str, ds_id: i64) -> Result<()> {
        let now = Utc::now().timestamp();
        self.conn.execute(sql::UPSERT_DOC_CHUNK, params![dik, value, now, ds_id])?;
        Ok(())
    }

    fn query_to_vec(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<String>>>().map_err(BookdbError::from)
    }

    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.query_to_vec(sql::LIST_PROJECTS, &[])
    }

    pub fn list_docstores(&self, p_id: i64) -> Result<Vec<String>> {
        self.query_to_vec(sql::LIST_DOCSTORES, &[&p_id])
    }

    pub fn list_varstores(&self, ds_id: i64) -> Result<Vec<String>> {
        self.query_to_vec(sql::LIST_VARSTORES, &[&ds_id])
    }

    pub fn list_keys(&self, vs_id: i64) -> Result<Vec<String>> {
        self.query_to_vec(sql::LIST_KEYS, &[&vs_id])
    }

    pub fn list_diks(&self, ds_id: i64) -> Result<Vec<String>> {
        self.query_to_vec(sql::LIST_DIKS, &[&ds_id])
    }

    fn query_to_kv_vec(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<(String, String)>>>().map_err(BookdbError::from)
    }

    pub fn stream_vars(&self, vs_id: i64) -> Result<Vec<(String, String)>> {
        self.query_to_kv_vec("SELECT var_key, var_value FROM vars WHERE vs_id_fk = ?1 ORDER BY var_key", &[&vs_id])
    }

    pub fn stream_doc_chunks(&self, ds_id: i64) -> Result<Vec<(String, String)>> {
        self.query_to_kv_vec("SELECT dc_key, dc_value FROM doc_chunks WHERE ds_id_fk = ?1 ORDER BY dc_key", &[&ds_id])
    }
}
