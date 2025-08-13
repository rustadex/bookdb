// src/db/docstore.rs - Document store database operations

use crate::error::Result;
use crate::context::ResolvedContext;
use crate::sql;
use rusqlite::{params, Transaction};
use super::Database;

impl Database {
    /// List documents in a workspace
    pub fn list_documents(&self, context: &ResolvedContext) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing documents in context: {}", context));
        
        let mut stmt = self.connection.prepare(sql::LIST_DOCUMENTS)?;
        let doc_iter = stmt.query_map([&context.project, &context.workspace], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut documents = Vec::new();
        for doc in doc_iter {
            documents.push(doc?);
        }
        
        Ok(documents)
    }
    
    /// Get document content by key and context
    pub fn get_document(&self, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        self.logger.trace_fn("database", &format!("getting document {} in context: {}", key, context));
        
        let mut stmt = self.connection.prepare(sql::GET_DOCUMENT)?;
        let mut rows = stmt.query_map([&context.project, &context.workspace, key], |row| {
            Ok(row.get::<_, Option<String>>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(content)) => Ok(content),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
    
    /// Set document content (upsert operation)
    pub fn set_document(&self, key: &str, content: &str, context: &ResolvedContext) ->  Result<(), E> {
        self.logger.trace_fn("database", &format!("setting document {} in context: {}", key, context));
        
        let tx = self.connection.transaction()?;
        
        // Ensure doc store exists
        let ds_id = self.ensure_doc_store_exists(&tx, context)?;
        
        // Upsert the document
        tx.execute(sql::SET_DOCUMENT, params![key, content, ds_id])?;
        
        tx.commit()?;
        Ok(())
    }
    
    /// Delete document by key and context
    pub fn delete_document(&self, key: &str, context: &ResolvedContext) -> Result<bool, E> {
        self.logger.trace_fn("database", &format!("deleting document {} in context: {}", key, context));
        
        let changes = self.connection.execute(
            sql::DELETE_DOCUMENT,
            params![&context.project, &context.workspace, key]
        )?;
        
        Ok(changes > 0)
    }
    
    /// Count documents in a workspace
    pub fn count_documents(&self, context: &ResolvedContext) -> Result<usize> {
        self.logger.trace_fn("database", &format!("counting documents in context: {}", context));
        
        let count: i64 = self.connection.query_row(
            sql::COUNT_DOCUMENTS,
            params![&context.project, &context.workspace],
            |row| Ok(row.get(0)?)
        )?;
        
        Ok(count as usize)
    }
    
    /// Get document segment content
    pub fn get_doc_segment(&self, doc_key: &str, path: &str) -> Result<Option<Vec<u8>>> {
        self.logger.trace_fn("database", &format!("getting doc segment {}.{}", doc_key, path));
        
        let mut stmt = self.connection.prepare(sql::GET_DOC_SEGMENT)?;
        let mut rows = stmt.query_map([doc_key, path], |row| {
            Ok(row.get::<_, Vec<u8>>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(content)) => Ok(Some(content)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
    
    /// Set document segment content
    pub fn set_doc_segment(&self, doc_key: &str, path: &str, mime: &str, content: &[u8], ds_id: i64) ->  Result<(), E> {
        self.logger.trace_fn("database", &format!("setting doc segment {}.{}", doc_key, path));
        
        let tx = self.connection.transaction()?;
        
        // Ensure document exists
        tx.execute(
            "INSERT OR IGNORE INTO docs (doc_key, ds_id_fk, doc_updated) VALUES (?1, ?2, strftime('%s','now'))",
            params![doc_key, ds_id]
        )?;
        
        // Get document ID
        let doc_id: i64 = tx.query_row(
            "SELECT doc_id FROM docs WHERE doc_key = ?1 AND ds_id_fk = ?2",
            params![doc_key, ds_id],
            |row| Ok(row.get(0)?)
        )?;
        
        // Set the segment
        tx.execute(sql::SET_DOC_SEGMENT, params![path, mime, content, doc_id])?;
        
        tx.commit()?;
        Ok(())
    }
    
    /// Ensure doc store exists, return doc store ID
    fn ensure_doc_store_exists(&self, tx: &Transaction, context: &ResolvedContext) -> Result<i64> {
        // First ensure project exists
        let project_id = self.ensure_project_exists_tx(tx, &context.project)?;
        
        // Try to get existing doc store
        let mut stmt = tx.prepare(sql::GET_DOC_STORE_ID)?;
        let mut rows = stmt.query_map([&context.project, &context.workspace], |row| {
            Ok(row.get::<_, i64>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(ds_id)) => Ok(ds_id),
            Some(Err(e)) => Err(e.into()),
            None => {
                // Create the doc store
                tx.execute(sql::ENSURE_DOC_STORE, params![project_id, &context.workspace])?;
                Ok(tx.last_insert_rowid())
            }
        }
    }
    
    /// Create a new document store (stub - implicit through first document)
    pub fn create_docstore(&self, _project: &str, _workspace: &str) ->  Result<(), E> {
        // TODO: Implement explicit docstore creation if needed
        todo!("Explicit docstore creation not yet implemented")
    }
    
    /// Delete a document store (stub - not implemented)
    pub fn delete_docstore(&self, _project: &str, _workspace: &str) ->  Result<(), E> {
        // TODO: Implement docstore deletion with cascade
        todo!("Docstore deletion not yet implemented")
    }
    
    /// List document stores in a workspace (stub - not implemented)
    pub fn list_docstores(&self, _project: &str, _workspace: &str) -> Result<Vec<String>> {
        // TODO: Implement docstore listing if multiple docstores per workspace are supported
        todo!("Docstore listing not yet implemented")
    }
}
