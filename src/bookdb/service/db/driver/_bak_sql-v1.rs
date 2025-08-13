// src/db.rs - Complete database implementation with atomic operations added

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
        let mut logger = Stderr::new();
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
    
    /// Set up database schema using external SQL files
    fn setup_schema(&mut self) ->  Result<(), E> {
        self.logger.trace_fn("database", "setting up schema from SQL files");
        
        // Execute schema creation from external SQL file
        self.connection.execute_batch(crate::sql::SqlQueries::CREATE_TABLES)?;
        
        self.logger.trace_fn("database", "schema setup complete");
        Ok(())
    }
    
    /// Validate that a string value represents a valid integer
    fn validate_numeric_value(&self, value: &str) -> Result<i64> {
        value.trim().parse::<i64>()
            .map_err(|_| BookdbError::NonNumericValue("unknown".to_string(), value.to_string()))
    }
    
    /// Atomically increment a variable by the specified amount
    pub fn increment_variable(&self, key: &str, amount: i64, context: &ResolvedContext) -> Result<i64> {
        self.logger.trace_fn("database", &format!("incrementing key '{}' by {} in context: {}", key, amount, context));
        
        let tx = self.connection.unchecked_transaction()?;
        
        // Get current value
        let current_value = match self.get_variable_in_transaction(&tx, key, context)? {
            Some(value) => {
                // Validate that current value is numeric
                self.validate_numeric_value(&value)
                    .map_err(|_| BookdbError::NonNumericValue(key.to_string(), value))?
            }
            None => {
                // Key doesn't exist, initialize to 0
                self.logger.trace_fn("database", &format!("key '{}' not found, initializing to 0", key));
                0
            }
        };
        
        // Check for overflow
        let new_value = current_value.checked_add(amount)
            .ok_or(BookdbError::NumericOverflow)?;
        
        // Set the new value
        self.set_variable_in_transaction(&tx, key, &new_value.to_string(), context)?;
        
        tx.commit()?;
        
        self.logger.trace_fn("database", &format!("incremented '{}': {} + {} = {}", key, current_value, amount, new_value));
        Ok(new_value)
    }
    
    /// Atomically decrement a variable by the specified amount
    pub fn decrement_variable(&self, key: &str, amount: i64, context: &ResolvedContext) -> Result<i64> {
        self.logger.trace_fn("database", &format!("decrementing key '{}' by {} in context: {}", key, amount, context));
        
        let tx = self.connection.unchecked_transaction()?;
        
        // Get current value
        let current_value = match self.get_variable_in_transaction(&tx, key, context)? {
            Some(value) => {
                // Validate that current value is numeric
                self.validate_numeric_value(&value)
                    .map_err(|_| BookdbError::NonNumericValue(key.to_string(), value))?
            }
            None => {
                // Key doesn't exist, initialize to 0
                self.logger.trace_fn("database", &format!("key '{}' not found, initializing to 0", key));
                0
            }
        };
        
        // Check for underflow
        let new_value = current_value.checked_sub(amount)
            .ok_or(BookdbError::NumericOverflow)?; // Using same error type for consistency
        
        // Set the new value
        self.set_variable_in_transaction(&tx, key, &new_value.to_string(), context)?;
        
        tx.commit()?;
        
        self.logger.trace_fn("database", &format!("decremented '{}': {} - {} = {}", key, current_value, amount, new_value));
        Ok(new_value)
    }
    
    /// Get variable within a transaction (helper for atomic operations)
    fn get_variable_in_transaction(&self, tx: &Transaction, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        let kvns_id = self.resolve_kvns_id_in_transaction(tx, context)?;
        
        let mut stmt = tx.prepare(
            "SELECT var_value FROM vars WHERE var_key = ? AND kvns_id_fk = ?"
        )?;
        
        let result = stmt.query_row(params![key, kvns_id], |row| {
            Ok(row.get::<_, Option<String>>(0)?)
        });
        
        match result {
            Ok(value) => Ok(value),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BookdbError::Sql(e)),
        }
    }
    
    /// Set variable within a transaction (helper for atomic operations)
    fn set_variable_in_transaction(&self, tx: &Transaction, key: &str, value: &str, context: &ResolvedContext) ->  Result<(), E> {
        let kvns_id = self.resolve_kvns_id_in_transaction(tx, context)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        tx.execute(
            "INSERT OR REPLACE INTO vars (var_key, var_value, var_updated, kvns_id_fk) VALUES (?, ?, ?, ?)",
            params![key, value, timestamp, kvns_id],
        )?;
        
        Ok(())
    }
    
    /// Resolve kvns_id within a transaction
    fn resolve_kvns_id_in_transaction(&self, tx: &Transaction, context: &ResolvedContext) -> Result<i64> {
        // Ensure project exists
        let pns_id = self.ensure_project_exists_in_transaction(tx, &context.project_name)?;
        
        // Ensure keyval namespace exists
        let kvns_id = self.ensure_kvns_exists_in_transaction(
            tx,
            pns_id,
            &context.workspace_name,
            &context.keystore_name,
        )?;
        
        Ok(kvns_id)
    }
    
    /// Ensure project exists within transaction and return its ID
    fn ensure_project_exists_in_transaction(&self, tx: &Transaction, project_name: &str) -> Result<i64> {
        // Try to get existing project
        let result = tx.query_row(
            "SELECT pns_id FROM project_ns WHERE pns_name = ?",
            params![project_name],
            |row| Ok(row.get::<_, i64>(0)?),
        );
        
        match result {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Create new project
                tx.execute(
                    "INSERT INTO project_ns (pns_name) VALUES (?)",
                    params![project_name],
                )?;
                Ok(tx.last_insert_rowid())
            }
            Err(e) => Err(BookdbError::Sql(e)),
        }
    }
    
    /// Ensure keyval namespace exists within transaction and return its ID
    fn ensure_kvns_exists_in_transaction(&self, tx: &Transaction, pns_id: i64, workspace_name: &str, keystore_name: &str) -> Result<i64> {
        // Try to get existing kvns
        let result = tx.query_row(
            "SELECT kvns_id FROM keyval_ns WHERE kvns_name = ? AND pns_id_fk = ? AND workspace_name = ?",
            params![keystore_name, pns_id, workspace_name],
            |row| Ok(row.get::<_, i64>(0)?),
        );
        
        match result {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Create new kvns
                tx.execute(
                    "INSERT INTO keyval_ns (kvns_name, pns_id_fk, workspace_name) VALUES (?, ?, ?)",
                    params![keystore_name, pns_id, workspace_name],
                )?;
                Ok(tx.last_insert_rowid())
            }
            Err(e) => Err(BookdbError::Sql(e)),
        }
    }
    
    /// Get variable value
    pub fn get_variable(&self, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        self.logger.trace_fn("database", &format!("getting variable: {} in context: {}", key, context));
        
        let tx = self.connection.unchecked_transaction()?;
        let result = self.get_variable_in_transaction(&tx, key, context)?;
        tx.commit()?;
        
        Ok(result)
    }
    
    /// Set variable value
    pub fn set_variable(&self, key: &str, value: &str, context: &ResolvedContext) ->  Result<(), E> {
        self.logger.trace_fn("database", &format!("setting variable: {}={} in context: {}", key, value, context));
        
        let tx = self.connection.unchecked_transaction()?;
        self.set_variable_in_transaction(&tx, key, value, context)?;
        tx.commit()?;
        
        Ok(())
    }
    
    /// Delete a variable
    pub fn delete_variable(&self, key: &str, context: &ResolvedContext) ->  Result<(), E> {
        self.logger.trace_fn("database", &format!("deleting variable: {} in context: {}", key, context));
        
        let tx = self.connection.unchecked_transaction()?;
        let kvns_id = self.resolve_kvns_id_in_transaction(&tx, context)?;
        
        let affected = tx.execute(
            "DELETE FROM vars WHERE var_key = ? AND kvns_id_fk = ?",
            params![key, kvns_id],
        )?;
        
        tx.commit()?;
        
        if affected == 0 {
            return Err(BookdbError::KeyNotFound(key.to_string()));
        }
        
        Ok(())
    }
    
    /// List all variables in a context
    pub fn list_variables(&self, context: &ResolvedContext) -> Result<HashMap<String, String>> {
        self.logger.trace_fn("database", &format!("listing variables in context: {}", context));
        
        let tx = self.connection.unchecked_transaction()?;
        let kvns_id = self.resolve_kvns_id_in_transaction(&tx, context)?;
        
        let mut stmt = tx.prepare(
            "SELECT var_key, var_value FROM vars WHERE kvns_id_fk = ? ORDER BY var_key"
        )?;
        
        let var_iter = stmt.query_map([kvns_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut variables = HashMap::new();
        for var_result in var_iter {
            let (key, value) = var_result?;
            variables.insert(key, value);
        }
        
        tx.commit()?;
        Ok(variables)
    }
    
    /// List all projects using external SQL
    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.logger.trace_fn("database", "listing all projects using SQL file");
        
        let mut stmt = self.connection.prepare(crate::sql::SqlQueries::LIST_PROJECTS)?;
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
    pub fn list_workspaces(&self, project: &str) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing workspaces in project: {}", project));
        
        let mut stmt = self.connection.prepare(
            "SELECT DISTINCT workspace_name FROM keyval_ns kvns 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ? ORDER BY workspace_name"
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
    
    /// List all keystores in a project/workspace
    pub fn list_keystores(&self, project: &str, workspace: &str) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing keystores in project: {}, workspace: {}", project, workspace));
        
        let mut stmt = self.connection.prepare(
            "SELECT kvns.kvns_name FROM keyval_ns kvns 
             JOIN project_ns pns ON kvns.pns_id_fk = pns.pns_id 
             WHERE pns.pns_name = ? AND kvns.workspace_name = ? ORDER BY kvns.kvns_name"
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
    
    /// Get document content
    pub fn get_document(&self, dik: &str, context: &ResolvedContext) -> Result<Option<String>> {
        self.logger.trace_fn("database", &format!("getting document: {} in context: {}", dik, context));
        
        // For now, return None - document functionality not fully implemented
        Ok(None)
    }
    
    /// Set document content
    pub fn set_document(&self, dik: &str, content: &str, context: &ResolvedContext) ->  Result<(), E> {
        self.logger.trace_fn("database", &format!("setting document: {}=<content> in context: {}", dik, context));
        
        // For now, just log - document functionality not fully implemented
        self.logger.info("Document operations not yet fully implemented");
        Ok(())
    }
    
    /// List documents in a context
    pub fn list_documents(&self, context: &ResolvedContext) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing documents in context: {}", context));
        
        // For now, return empty list - document functionality not fully implemented
        Ok(Vec::new())
    }
    
    /// Export data from context with filters
    pub fn export_data(&self, context: &ResolvedContext, filters: &(Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>)) -> Result<Vec<ExportItem>> {
        self.logger.trace_fn("database", &format!("exporting data from context: {}", context));
        
        let (proj_filter, workspace_filter, keystore_filter, doc_filter, key_filter, _seg_filter) = filters;
        
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
    pub fn import_variables(&self, variables: HashMap<String, String>, context: &ResolvedContext) ->  Result<(), E> {
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
    pub fn execute_sql(&self, sql: &str) ->  Result<(), E> {
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
    use crate::context::ResolvedContext;
    
    fn create_test_context() -> ResolvedContext {
        ResolvedContext {
            base_name: "test".to_string(),
            project_name: "myapp".to_string(),
            workspace_name: "config".to_string(),
            keystore_name: "settings".to_string(),
        }
    }
    
    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::create_or_open(&db_path).unwrap();
        (db, temp_dir)
    }
    
    #[test]
    fn test_variable_operations() ->  Result<(), E> {
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
    fn test_increment_operations() ->  Result<(), E> {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test incrementing new key
        let result = db.increment_variable("COUNTER", 5, &context)?;
        assert_eq!(result, 5);
        
        // Test incrementing existing key
        let result = db.increment_variable("COUNTER", 3, &context)?;
        assert_eq!(result, 8);
        
        // Test decrementing
        let result = db.decrement_variable("COUNTER", 2, &context)?;
        assert_eq!(result, 6);
        
        Ok(())
    }
    
    #[test]
    fn test_project_operations() ->  Result<(), E> {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Create a project by setting a variable (which creates the hierarchy)
        db.set_variable("TEST_KEY", "test_value", &context)?;
        
        // Test listing projects
        let projects = db.list_projects()?;
        assert!(projects.contains(&"myapp".to_string()));
        
        Ok(())
    }
}
