// src/db.rs - Updated with consistent BOOKDB_CONCEPTS.md terminology

use crate::error::{Result, BookdbError};
use crate::context::{ResolvedContext, Anchor};
use crate::rdx::stderr::{Stderr, StderrConfig};
use rusqlite::{params, Connection, Transaction, Row};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Database connection with stderr integration
pub struct Database {
    connection: Connection,
    logger: Stderr,
    base_name: String,
}

impl Database {
    /// Create or open a database at the specified path
    pub fn create_or_open(path: &Path) -> Result<Self> {
        let mut logger = Stderr::new(&StderrConfig::from_env());
        logger.trace_fn("database", &format!("opening database: {:?}", path));
        
        let connection = Connection::open(path)?;
        
        // Enable foreign keys
        connection.execute("PRAGMA foreign_keys = ON", [])?;
        
        let base_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let mut db = Self {
            connection,
            logger,
            base_name,
        };
        
        db.setup_schema()?;
        logger.trace_fn("database", "database ready");
        
        Ok(db)
    }
    
    /// Open existing database (installation guard ensures it exists)
    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(BookdbError::Database(format!("Database not found: {:?}", path)));
        }
        Self::create_or_open(path)
    }
    
    /// Set up database schema matching bash version exactly
    fn setup_schema(&mut self) -> Result<()> {
        self.logger.trace_fn("database", "setting up schema");
        
        // Project namespace table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS project_ns (
                pns_id INTEGER PRIMARY KEY,
                pns_name TEXT UNIQUE NOT NULL
            )",
            [],
        )?;
        
        // Keyval namespace table (workspace.var.keystore structure)
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS keyval_ns (
                kvns_id INTEGER PRIMARY KEY,
                kvns_name TEXT NOT NULL,
                pns_id_fk INTEGER,
                workspace_name TEXT NOT NULL DEFAULT 'GLOBAL',
                FOREIGN KEY(pns_id_fk) REFERENCES project_ns(pns_id) ON DELETE CASCADE,
                UNIQUE(kvns_name, pns_id_fk, workspace_name)
            )",
            [],
        )?;
        
        // Variables table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS vars (
                var_id INTEGER PRIMARY KEY,
                var_key TEXT NOT NULL,
                var_value TEXT,
                var_updated INTEGER DEFAULT (strftime('%s','now')),
                kvns_id_fk INTEGER,
                FOREIGN KEY(kvns_id_fk) REFERENCES keyval_ns(kvns_id) ON DELETE CASCADE,
                UNIQUE(var_key, kvns_id_fk)
            )",
            [],
        )?;
        
        // Document stores table (for workspace.doc structure)
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS doc_stores (
                ds_id INTEGER PRIMARY KEY,
                ds_name TEXT NOT NULL,
                pns_id_fk INTEGER,
                workspace_name TEXT NOT NULL DEFAULT 'GLOBAL',
                FOREIGN KEY(pns_id_fk) REFERENCES project_ns(pns_id) ON DELETE CASCADE,
                UNIQUE(ds_name, pns_id_fk, workspace_name)
            )",
            [],
        )?;
        
        // Documents table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS docs (
                doc_id INTEGER PRIMARY KEY,
                doc_key TEXT NOT NULL,
                doc_content TEXT,
                doc_updated INTEGER DEFAULT (strftime('%s','now')),
                ds_id_fk INTEGER,
                FOREIGN KEY(ds_id_fk) REFERENCES doc_stores(ds_id) ON DELETE CASCADE,
                UNIQUE(doc_key, ds_id_fk)
            )",
            [],
        )?;
        
        self.logger.trace_fn("database", "schema setup complete");
        Ok(())
    }
    
    /// List all projects in the database
    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.logger.trace_fn("database", "listing all projects");
        
        let mut stmt = self.connection.prepare("SELECT pns_name FROM project_ns ORDER BY pns_name")?;
        let project_iter = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }
        
        Ok(projects)
    }
    
    /// List all workspaces in a project
    pub fn list_workspaces(&self, project: &str) -> Result<Vec<String>> {  // FIXED: was list_docstores
        self.logger.trace_fn("database", &format!("listing workspaces in project: {}", project));
        
        // Get distinct workspace names from keyval_ns for this project
        let mut stmt = self.connection.prepare(
            "SELECT DISTINCT workspace_name 
             FROM keyval_ns kvns 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 
             ORDER BY workspace_name"
        )?;
        
        let workspace_iter = stmt.query_map([project], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut workspaces = Vec::new();
        for workspace in workspace_iter {
            workspaces.push(workspace?);
        }
        
        Ok(workspaces)
    }
    
    /// List all keystores in a workspace within a project  
    pub fn list_keystores(&self, project: &str, workspace: &str) -> Result<Vec<String>> {  // FIXED: was list_varstores
        self.logger.trace_fn("database", &format!("listing keystores in {}.{}", project, workspace));
        
        let mut stmt = self.connection.prepare(
            "SELECT kvns.kvns_name 
             FROM keyval_ns kvns 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 AND kvns.workspace_name = ?2 
             ORDER BY kvns.kvns_name"
        )?;
        
        let keystore_iter = stmt.query_map([project, workspace], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut keystores = Vec::new();
        for keystore in keystore_iter {
            keystores.push(keystore?);
        }
        
        Ok(keystores)
    }
    
    /// List all variables in a context
    pub fn list_variables(&self, context: &ResolvedContext) -> Result<HashMap<String, String>> {
        self.logger.trace_fn("database", &format!("listing variables in context: {}", context));
        
        let mut stmt = self.connection.prepare(
            "SELECT v.var_key, v.var_value 
             FROM vars v 
             JOIN keyval_ns kvns ON v.kvns_id_fk = kvns.kvns_id 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 AND kvns.workspace_name = ?2 AND kvns.kvns_name = ?3 
             ORDER BY v.var_key"
        )?;
        
        let var_iter = stmt.query_map([&context.project, &context.workspace, &context.tail], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut variables = HashMap::new();
        for var_result in var_iter {
            let (key, value) = var_result?;
            variables.insert(key, value);
        }
        
        Ok(variables)
    }
    
    /// Get a variable value
    pub fn get_variable(&self, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        self.logger.trace_fn("database", &format!("getting variable: {} in context: {}", key, context));
        
        let result = self.connection.query_row(
            "SELECT v.var_value 
             FROM vars v 
             JOIN keyval_ns kvns ON v.kvns_id_fk = kvns.kvns_id 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 AND kvns.workspace_name = ?2 AND kvns.kvns_name = ?3 AND v.var_key = ?4",
            params![&context.project, &context.workspace, &context.tail, key],
            |row| Ok(row.get::<_, String>(0)?)
        );
        
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BookdbError::Database(e.to_string())),
        }
    }
    
    /// Set a variable value
    pub fn set_variable(&self, key: &str, value: &str, context: &ResolvedContext) -> Result<()> {
        self.logger.trace_fn("database", &format!("setting variable: {}={} in context: {}", key, value, context));
        
        // Ensure the namespace hierarchy exists
        self.ensure_keystore_exists(&context.project, &context.workspace, &context.tail)?;
        
        // Get the keystore ID
        let kvns_id: i64 = self.connection.query_row(
            "SELECT kvns.kvns_id 
             FROM keyval_ns kvns 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 AND kvns.workspace_name = ?2 AND kvns.kvns_name = ?3",
            params![&context.project, &context.workspace, &context.tail],
            |row| Ok(row.get(0)?)
        )?;
        
        // Insert or update the variable
        self.connection.execute(
            "INSERT OR REPLACE INTO vars (var_key, var_value, kvns_id_fk, var_updated) 
             VALUES (?1, ?2, ?3, strftime('%s','now'))",
            params![key, value, kvns_id],
        )?;
        
        Ok(())
    }
    
    /// Delete a variable
    pub fn delete_variable(&self, key: &str, context: &ResolvedContext) -> Result<()> {
        self.logger.trace_fn("database", &format!("deleting variable: {} in context: {}", key, context));
        
        let rows_affected = self.connection.execute(
            "DELETE FROM vars 
             WHERE var_key = ?4 AND kvns_id_fk = (
                 SELECT kvns.kvns_id 
                 FROM keyval_ns kvns 
                 JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
                 WHERE pns.pns_name = ?1 AND kvns.workspace_name = ?2 AND kvns.kvns_name = ?3
             )",
            params![&context.project, &context.workspace, &context.tail, key],
        )?;
        
        if rows_affected == 0 {
            return Err(BookdbError::KeyNotFound(format!("Variable '{}' not found in context {}", key, context)));
        }
        
        Ok(())
    }
    
    /// Get a document
    pub fn get_document(&self, doc_key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        self.logger.trace_fn("database", &format!("getting document: {} in context: {}", doc_key, context));
        
        let result = self.connection.query_row(
            "SELECT d.doc_content 
             FROM docs d 
             JOIN doc_stores ds ON d.ds_id_fk = ds.ds_id 
             JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 AND ds.workspace_name = ?2 AND d.doc_key = ?3",
            params![&context.project, &context.workspace, doc_key],
            |row| Ok(row.get::<_, String>(0)?)
        );
        
        match result {
            Ok(content) => Ok(Some(content)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BookdbError::Database(e.to_string())),
        }
    }
    
    /// Set a document
    pub fn set_document(&self, doc_key: &str, content: &str, context: &ResolvedContext) -> Result<()> {
        self.logger.trace_fn("database", &format!("setting document: {} in context: {}", doc_key, context));
        
        // Ensure the workspace exists for documents
        self.ensure_workspace_exists(&context.project, &context.workspace)?;
        
        // Get or create document store ID
        let ds_id = self.get_or_create_doc_store(&context.project, &context.workspace)?;
        
        // Insert or update the document
        self.connection.execute(
            "INSERT OR REPLACE INTO docs (doc_key, doc_content, ds_id_fk, doc_updated) 
             VALUES (?1, ?2, ?3, strftime('%s','now'))",
            params![doc_key, content, ds_id],
        )?;
        
        Ok(())
    }
    
    /// List documents in a workspace
    pub fn list_documents(&self, context: &ResolvedContext) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing documents in context: {}", context));
        
        let mut stmt = self.connection.prepare(
            "SELECT d.doc_key 
             FROM docs d 
             JOIN doc_stores ds ON d.ds_id_fk = ds.ds_id 
             JOIN project_ns pns ON ds.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ?1 AND ds.workspace_name = ?2 
             ORDER BY d.doc_key"
        )?;
        
        let doc_iter = stmt.query_map([&context.project, &context.workspace], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut documents = Vec::new();
        for doc in doc_iter {
            documents.push(doc?);
        }
        
        Ok(documents)
    }
    
    /// Ensure a project exists in the database
    pub fn ensure_project_exists(&self, project: &str) -> Result<()> {
        self.logger.trace_fn("database", &format!("ensuring project exists: {}", project));
        
        self.connection.execute(
            "INSERT OR IGNORE INTO project_ns (pns_name) VALUES (?1)",
            params![project],
        )?;
        
        Ok(())
    }
    
    /// Ensure a workspace exists within a project
    pub fn ensure_workspace_exists(&self, project: &str, workspace: &str) -> Result<()> {  // FIXED: was ensure_docstore_exists
        self.logger.trace_fn("database", &format!("ensuring workspace exists: {}.{}", project, workspace));
        
        // First ensure project exists
        self.ensure_project_exists(project)?;
        
        // Workspace existence is implicit when keystores or doc stores are created
        // No explicit workspace table - workspaces are defined by the workspace_name field
        
        Ok(())
    }
    
    /// Ensure a keystore exists within a workspace
    pub fn ensure_keystore_exists(&self, project: &str, workspace: &str, keystore: &str) -> Result<()> {  // FIXED: was ensure_varstore_exists
        self.logger.trace_fn("database", &format!("ensuring keystore exists: {}.{}.{}", project, workspace, keystore));
        
        // First ensure project exists
        self.ensure_project_exists(project)?;
        
        // Get project ID
        let project_id: i64 = self.connection.query_row(
            "SELECT pns_id FROM project_ns WHERE pns_name = ?1",
            params![project],
            |row| Ok(row.get(0)?)
        )?;
        
        // Create keystore (keyval_ns) with workspace name
        self.connection.execute(
            "INSERT OR IGNORE INTO keyval_ns (kvns_name, pns_id_fk, workspace_name) VALUES (?1, ?2, ?3)",
            params![keystore, project_id, workspace],
        )?;
        
        Ok(())
    }
    
    /// Get or create a document store for a workspace
    fn get_or_create_doc_store(&self, project: &str, workspace: &str) -> Result<i64> {
        self.logger.trace_fn("database", &format!("getting doc store for {}.{}", project, workspace));
        
        // First ensure project exists
        self.ensure_project_exists(project)?;
        
        // Get project ID
        let project_id: i64 = self.connection.query_row(
            "SELECT pns_id FROM project_ns WHERE pns_name = ?1",
            params![project],
            |row| Ok(row.get(0)?)
        )?;
        
        // Try to get existing doc store
        let ds_result = self.connection.query_row(
            "SELECT ds_id FROM doc_stores WHERE pns_id_fk = ?1 AND workspace_name = ?2",
            params![project_id, workspace],
            |row| Ok(row.get::<_, i64>(0)?)
        );
        
        match ds_result {
            Ok(ds_id) => Ok(ds_id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Create new doc store
                self.connection.execute(
                    "INSERT INTO doc_stores (ds_name, pns_id_fk, workspace_name) VALUES (?1, ?2, ?3)",
                    params!["documents", project_id, workspace],
                )?;
                
                Ok(self.connection.last_insert_rowid())
            }
            Err(e) => Err(BookdbError::Database(e.to_string())),
        }
    }
    
    /// Export data from context with optional filters
    pub fn export_data(&self, context: &ResolvedContext, filters: &(Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>)) -> Result<Vec<ExportItem>> {
        self.logger.trace_fn("database", &format!("exporting data from context: {}", context));
        
        let (proj_filter, workspace_filter, keystore_filter, doc_filter, key_filter, _seg_filter) = filters;
        // FIXED: parameter names updated from ds_filter, vs_filter
        
        let mut items = Vec::new();
        
        // Export variables
        let variables = self.list_variables(context)?;
        for (key, value) in variables {
            // Apply key filter if specified
            if let Some(key_pattern) = key_filter {
                if !key.contains(key_pattern) {
                    continue;
                }
            }
            
            items.push(ExportItem {
                item_type: "variable".to_string(),
                key,
                value,
                context: format!("{}", context),
            });
        }
        
        Ok(items)
    }
    
    /// Import variables into context
    pub fn import_variables(&self, variables: HashMap<String, String>, context: &ResolvedContext) -> Result<()> {
        self.logger.trace_fn("database", &format!("importing {} variables into context: {}", variables.len(), context));
        
        for (key, value) in variables {
            self.set_variable(&key, &value, context)?;
        }
        
        Ok(())
    }
    
    /// Migrate legacy data (placeholder)
    pub fn migrate_legacy_data(&self, context: &ResolvedContext, dry_run: bool) -> Result<usize> {
        self.logger.trace_fn("database", &format!("migrating legacy data in context: {} (dry_run: {})", context, dry_run));
        
        // TODO: Implement actual migration logic
        // For now, return 0 as no legacy data to migrate
        Ok(0)
    }
    
    /// Execute raw SQL (for installation and setup)
    pub fn execute_sql(&self, sql: &str) -> Result<()> {
        self.logger.trace_fn("database", "executing raw SQL");
        self.connection.execute(sql, [])?;
        Ok(())
    }
}

/// Data structure for export operations
#[derive(Debug, Clone)]
pub struct ExportItem {
    pub item_type: String,
    pub key: String,
    pub value: String,
    pub context: String,
}

impl serde::Serialize for ExportItem {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ExportItem", 4)?;
        state.serialize_field("type", &self.item_type)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("context", &self.context)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::context::{Anchor, ChainMode};
    
    fn create_test_context() -> ResolvedContext {
        ResolvedContext {
            base: "test".to_string(),
            project: "myapp".to_string(),
            workspace: "config".to_string(),     // FIXED: consistent terminology
            anchor: Anchor::Var,
            tail: "settings".to_string(),
            prefix_mode: ChainMode::Persistent,
        }
    }
    
    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::create_or_open(&db_path).unwrap();
        (db, temp_dir)
    }
    
    #[test]
    fn test_variable_operations() -> Result<()> {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test setting a variable
        db.set_variable("TEST_KEY", "test_value", &context)?;
        
        // Test getting the variable
        let value = db.get_variable("TEST_KEY", &context)?;
        assert_eq!(value, Some("test_value".to_string()));
        
        // Test listing variables
        let variables = db.list_variables(&context)?;
        assert!(variables.contains_key("TEST_KEY"));
        
        // Test deleting the variable
        db.delete_variable("TEST_KEY", &context)?;
        let value = db.get_variable("TEST_KEY", &context)?;
        assert_eq!(value, None);
        
        Ok(())
    }
    
    #[test]
    fn test_workspace_operations() -> Result<()> {  // FIXED: test name
        let (db, _temp) = create_test_db();
        
        // Test listing workspaces
        let workspaces = db.list_workspaces("myapp")?;
        // Should be empty initially
        assert!(workspaces.is_empty() || workspaces.contains(&"config".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_keystore_operations() -> Result<()> {  // FIXED: test name was test_varstore_operations
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Create a keystore by setting a variable
        db.set_variable("TEST_KEY", "test_value", &context)?;
        
        // Test listing keystores
        let keystores = db.list_keystores("myapp", "config")?;
        assert!(keystores.contains(&"settings".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_project_operations() -> Result<()> {
        let (db, _temp) = create_test_db();
        
        // Test ensuring project exists
        db.ensure_project_exists("testproject")?;
        
        // Test listing projects
        let projects = db.list_projects()?;
        assert!(projects.contains(&"testproject".to_string()));
        
        Ok(())
    }
}
