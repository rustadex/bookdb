// src/db.rs - Complete database layer with stderr integration
//
// FEATURES:
// 1. SQLite connection management with proper schema
// 2. CRUD operations for variables with rich stderr output
// 3. Context-aware database operations
// 4. Batch operations for import/export
// 5. Safety features matching bash version

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
        
        // Keyval namespace table (workspace.var.keystore)
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
                var_updated INTEGER,
                kvns_id_fk INTEGER,
                FOREIGN KEY(kvns_id_fk) REFERENCES keyval_ns(kvns_id) ON DELETE CASCADE,
                UNIQUE(var_key, kvns_id_fk)
            )",
            [],
        )?;
        
        // Document table (for future doc support)
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS docs (
                doc_id INTEGER PRIMARY KEY,
                doc_key TEXT NOT NULL,
                doc_content TEXT,
                doc_updated INTEGER,
                kvns_id_fk INTEGER,
                FOREIGN KEY(kvns_id_fk) REFERENCES keyval_ns(kvns_id) ON DELETE CASCADE,
                UNIQUE(doc_key, kvns_id_fk)
            )",
            [],
        )?;
        
        // Meta table for installation tracking
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        self.logger.trace_fn("database", "schema setup complete");
        Ok(())
    }
    
    /// Ensure project exists, create if necessary
    pub fn ensure_project_exists(&mut self, project_name: &str) -> Result<i64> {
        self.logger.trace_fn("database", &format!("ensuring project exists: {}", project_name));
        
        // Insert or ignore
        self.connection.execute(
            "INSERT OR IGNORE INTO project_ns (pns_name) VALUES (?1)",
            params![project_name],
        )?;
        
        // Get the ID
        let id: i64 = self.connection.query_row(
            "SELECT pns_id FROM project_ns WHERE pns_name = ?1",
            params![project_name],
            |row| row.get(0),
        )?;
        
        self.logger.trace_fn("database", &format!("project {} has ID {}", project_name, id));
        Ok(id)
    }
    
    /// Ensure workspace exists within project
    pub fn ensure_workspace_exists(&mut self, project_name: &str, workspace_name: &str) -> Result<()> {
        let project_id = self.ensure_project_exists(project_name)?;
        
        self.logger.trace_fn("database", &format!("ensuring workspace exists: {}.{}", project_name, workspace_name));
        
        // For now, workspace is implicit in keyval_ns table
        // Future enhancement will add dedicated workspace table
        
        Ok(())
    }
    
    /// Ensure varstore exists (keyval_ns entry)
    pub fn ensure_varstore_exists(&mut self, project_name: &str, workspace_name: &str, varstore_name: &str) -> Result<i64> {
        let project_id = self.ensure_project_exists(project_name)?;
        
        self.logger.trace_fn("database", &format!("ensuring varstore exists: {}.{}.var.{}", project_name, workspace_name, varstore_name));
        
        // Insert or ignore
        self.connection.execute(
            "INSERT OR IGNORE INTO keyval_ns (kvns_name, pns_id_fk, workspace_name) VALUES (?1, ?2, ?3)",
            params![varstore_name, project_id, workspace_name],
        )?;
        
        // Get the ID
        let id: i64 = self.connection.query_row(
            "SELECT kvns_id FROM keyval_ns WHERE kvns_name = ?1 AND pns_id_fk = ?2 AND workspace_name = ?3",
            params![varstore_name, project_id, workspace_name],
            |row| row.get(0),
        )?;
        
        self.logger.trace_fn("database", &format!("varstore {}.{}.var.{} has ID {}", project_name, workspace_name, varstore_name, id));
        Ok(id)
    }
    
    /// Get variable value in resolved context
    pub fn get_variable(&self, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        if !matches!(context.anchor, Anchor::Var) {
            return Err(BookdbError::InvalidContext("get_variable requires VAR anchor".to_string()));
        }
        
        self.logger.trace_fn("database", &format!("getting variable: {} in context {}", key, context));
        
        let query = "
            SELECT v.var_value 
            FROM vars v 
            JOIN keyval_ns k ON v.kvns_id_fk = k.kvns_id 
            JOIN project_ns p ON k.pns_id_fk = p.pns_id 
            WHERE v.var_key = ?1 
              AND p.pns_name = ?2 
              AND k.workspace_name = ?3 
              AND k.kvns_name = ?4
        ";
        
        let result = self.connection.query_row(
            query,
            params![key, context.project, context.workspace, context.tail],
            |row| Ok(row.get::<_, String>(0)?),
        );
        
        match result {
            Ok(value) => {
                self.logger.trace_fn("database", &format!("found value for key: {}", key));
                Ok(Some(value))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                self.logger.trace_fn("database", &format!("key not found: {}", key));
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Set variable value in resolved context
    pub fn set_variable(&mut self, key: &str, value: &str, context: &ResolvedContext) -> Result<()> {
        if !matches!(context.anchor, Anchor::Var) {
            return Err(BookdbError::InvalidContext("set_variable requires VAR anchor".to_string()));
        }
        
        self.logger.trace_fn("database", &format!("setting variable: {}={} in context {}", key, value, context));
        
        // Ensure the full context exists
        let varstore_id = self.ensure_varstore_exists(&context.project, &context.workspace, &context.tail)?;
        
        let timestamp = chrono::Utc::now().timestamp();
        
        // Use INSERT OR REPLACE for upsert behavior
        self.connection.execute(
            "INSERT OR REPLACE INTO vars (var_key, var_value, var_updated, kvns_id_fk) 
             VALUES (?1, ?2, ?3, ?4)",
            params![key, value, timestamp, varstore_id],
        )?;
        
        self.logger.trace_fn("database", &format!("variable set: {}={}", key, value));
        Ok(())
    }
    
    /// Delete variable in resolved context
    pub fn delete_variable(&mut self, key: &str, context: &ResolvedContext) -> Result<bool> {
        if !matches!(context.anchor, Anchor::Var) {
            return Err(BookdbError::InvalidContext("delete_variable requires VAR anchor".to_string()));
        }
        
        self.logger.trace_fn("database", &format!("deleting variable: {} in context {}", key, context));
        
        let query = "
            DELETE FROM vars 
            WHERE var_key = ?1 
              AND kvns_id_fk = (
                  SELECT k.kvns_id 
                  FROM keyval_ns k 
                  JOIN project_ns p ON k.pns_id_fk = p.pns_id 
                  WHERE p.pns_name = ?2 
                    AND k.workspace_name = ?3 
                    AND k.kvns_name = ?4
              )
        ";
        
        let rows_affected = self.connection.execute(
            query,
            params![key, context.project, context.workspace, context.tail],
        )?;
        
        let deleted = rows_affected > 0;
        if deleted {
            self.logger.trace_fn("database", &format!("variable deleted: {}", key));
        } else {
            self.logger.trace_fn("database", &format!("variable not found for deletion: {}", key));
        }
        
        Ok(deleted)
    }
    
    /// List all variables in resolved context
    pub fn list_variables(&self, context: &ResolvedContext) -> Result<HashMap<String, String>> {
        if !matches!(context.anchor, Anchor::Var) {
            return Err(BookdbError::InvalidContext("list_variables requires VAR anchor".to_string()));
        }
        
        self.logger.trace_fn("database", &format!("listing variables in context {}", context));
        
        let query = "
            SELECT v.var_key, v.var_value 
            FROM vars v 
            JOIN keyval_ns k ON v.kvns_id_fk = k.kvns_id 
            JOIN project_ns p ON k.pns_id_fk = p.pns_id 
            WHERE p.pns_name = ?1 
              AND k.workspace_name = ?2 
              AND k.kvns_name = ?3
            ORDER BY v.var_key
        ";
        
        let mut stmt = self.connection.prepare(query)?;
        let var_iter = stmt.query_map(
            params![context.project, context.workspace, context.tail],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                ))
            },
        )?;
        
        let mut variables = HashMap::new();
        for var_result in var_iter {
            let (key, value) = var_result?;
            variables.insert(key, value);
        }
        
        self.logger.trace_fn("database", &format!("found {} variables", variables.len()));
        Ok(variables)
    }
    
    /// List all projects
    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.logger.trace_fn("database", "listing projects");
        
        let mut stmt = self.connection.prepare("SELECT pns_name FROM project_ns ORDER BY pns_name")?;
        let project_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;
        
        let mut projects = Vec::new();
        for project_result in project_iter {
            projects.push(project_result?);
        }
        
        self.logger.trace_fn("database", &format!("found {} projects", projects.len()));
        Ok(projects)
    }
    
    /// List all workspaces in a project
    pub fn list_workspaces(&self, project_name: &str) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing workspaces in project {}", project_name));
        
        let query = "
            SELECT DISTINCT k.workspace_name 
            FROM keyval_ns k 
            JOIN project_ns p ON k.pns_id_fk = p.pns_id 
            WHERE p.pns_name = ?1 
            ORDER BY k.workspace_name
        ";
        
        let mut stmt = self.connection.prepare(query)?;
        let workspace_iter = stmt.query_map(params![project_name], |row| Ok(row.get::<_, String>(0)?))?;
        
        let mut workspaces = Vec::new();
        for workspace_result in workspace_iter {
            workspaces.push(workspace_result?);
        }
        
        self.logger.trace_fn("database", &format!("found {} workspaces", workspaces.len()));
        Ok(workspaces)
    }
    
    /// List all varstores in a workspace
    pub fn list_varstores(&self, project_name: &str, workspace_name: &str) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing varstores in {}.{}", project_name, workspace_name));
        
        let query = "
            SELECT k.kvns_name 
            FROM keyval_ns k 
            JOIN project_ns p ON k.pns_id_fk = p.pns_id 
            WHERE p.pns_name = ?1 AND k.workspace_name = ?2 
            ORDER BY k.kvns_name
        ";
        
        let mut stmt = self.connection.prepare(query)?;
        let varstore_iter = stmt.query_map(params![project_name, workspace_name], |row| Ok(row.get::<_, String>(0)?))?;
        
        let mut varstores = Vec::new();
        for varstore_result in varstore_iter {
            varstores.push(varstore_result?);
        }
        
        self.logger.trace_fn("database", &format!("found {} varstores", varstores.len()));
        Ok(varstores)
    }
    
    /// Execute raw SQL for advanced operations
    pub fn execute_sql(&mut self, sql: &str) -> Result<usize> {
        self.logger.trace_fn("database", &format!("executing SQL: {}", sql));
        let rows_affected = self.connection.execute(sql, [])?;
        self.logger.trace_fn("database", &format!("SQL affected {} rows", rows_affected));
        Ok(rows_affected)
    }
    
    /// Set variable in specific context chain (for installation)
    pub fn set_variable_in_context(&mut self, key: &str, value: &str, context: &crate::context::ContextChain) -> Result<()> {
        use crate::context::DefaultResolver;
        
        // Convert ContextChain to ResolvedContext
        let resolved = ResolvedContext {
            base: context.base.as_ref().unwrap_or(&self.base_name).clone(),
            project: context.project.clone(),
            workspace: context.workspace.clone(),
            anchor: context.anchor,
            tail: context.tail.clone(),
            prefix_mode: context.prefix_mode,
        };
        
        self.set_variable(key, value, &resolved)
    }
    
    /// Get variable in specific context chain (for installation verification)
    pub fn get_variable_in_context(&self, key: &str, context: &crate::context::ContextChain) -> Result<Option<String>> {
        use crate::context::DefaultResolver;
        
        // Convert ContextChain to ResolvedContext  
        let resolved = ResolvedContext {
            base: context.base.as_ref().unwrap_or(&self.base_name).clone(),
            project: context.project.clone(),
            workspace: context.workspace.clone(),
            anchor: context.anchor,
            tail: context.tail.clone(),
            prefix_mode: context.prefix_mode,
        };
        
        self.get_variable(key, &resolved)
    }
    
    /// Import variables from map with progress tracking
    pub fn import_variables(&mut self, variables: HashMap<String, String>, context: &ResolvedContext) -> Result<()> {
        if !matches!(context.anchor, Anchor::Var) {
            return Err(BookdbError::InvalidContext("import_variables requires VAR anchor".to_string()));
        }
        
        self.logger.trace_fn("database", &format!("importing {} variables to context {}", variables.len(), context));
        
        // Use transaction for batch operation
        let tx = self.connection.transaction()?;
        
        // Ensure varstore exists
        let varstore_id = self.ensure_varstore_exists(&context.project, &context.workspace, &context.tail)?;
        
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO vars (var_key, var_value, var_updated, kvns_id_fk) 
                 VALUES (?1, ?2, ?3, ?4)"
            )?;
            
            let timestamp = chrono::Utc::now().timestamp();
            
            for (key, value) in variables {
                stmt.execute(params![key, value, timestamp, varstore_id])?;
            }
        }
        
        tx.commit()?;
        self.logger.trace_fn("database", "import complete");
        Ok(())
    }
    
    /// Export all variables from context
    pub fn export_variables(&self, context: &ResolvedContext) -> Result<HashMap<String, String>> {
        self.list_variables(context)
    }
    
    /// Find variables matching pattern across all contexts
    pub fn find_variables(&self, pattern: &str) -> Result<Vec<(String, String, String, String)>> {
        self.logger.trace_fn("database", &format!("finding variables matching pattern: {}", pattern));
        
        let search_pattern = if pattern.contains('%') {
            pattern.to_string()
        } else {
            format!("%{}%", pattern)
        };
        
        let query = "
            SELECT p.pns_name, k.workspace_name, k.kvns_name, v.var_key 
            FROM vars v 
            JOIN keyval_ns k ON v.kvns_id_fk = k.kvns_id 
            JOIN project_ns p ON k.pns_id_fk = p.pns_id 
            WHERE v.var_key LIKE ?1 
            ORDER BY p.pns_name, k.workspace_name, k.kvns_name, v.var_key
        ";
        
        let mut stmt = self.connection.prepare(query)?;
        let result_iter = stmt.query_map(params![search_pattern], |row| {
            Ok((
                row.get::<_, String>(0)?, // project
                row.get::<_, String>(1)?, // workspace
                row.get::<_, String>(2)?, // varstore
                row.get::<_, String>(3)?, // key
            ))
        })?;
        
        let mut results = Vec::new();
        for result in result_iter {
            results.push(result?);
        }
        
        self.logger.trace_fn("database", &format!("found {} matching variables", results.len()));
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::context::{ContextChain, Anchor, ChainMode};
    
    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::create_or_open(&db_path).unwrap();
        (db, temp_dir)
    }
    
    fn create_test_context() -> ResolvedContext {
        ResolvedContext {
            base: "test".to_string(),
            project: "myapp".to_string(),
            workspace: "config".to_string(),
            anchor: Anchor::Var,
            tail: "settings".to_string(),
            prefix_mode: ChainMode::Persistent,
        }
    }
    
    #[test]
    fn test_variable_crud() -> Result<()> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test set
        db.set_variable("API_KEY", "secret123", &context)?;
        
        // Test get
        let value = db.get_variable("API_KEY", &context)?;
        assert_eq!(value, Some("secret123".to_string()));
        
        // Test update
        db.set_variable("API_KEY", "newsecret456", &context)?;
        let updated_value = db.get_variable("API_KEY", &context)?;
        assert_eq!(updated_value, Some("newsecret456".to_string()));
        
        // Test delete
        let deleted = db.delete_variable("API_KEY", &context)?;
        assert!(deleted);
        
        // Test get after delete
        let value_after_delete = db.get_variable("API_KEY", &context)?;
        assert_eq!(value_after_delete, None);
        
        Ok(())
    }
    
    #[test]
    fn test_list_operations() -> Result<()> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set multiple variables
        db.set_variable("KEY1", "value1", &context)?;
        db.set_variable("KEY2", "value2", &context)?;
        db.set_variable("KEY3", "value3", &context)?;
        
        // Test list variables
        let variables = db.list_variables(&context)?;
        assert_eq!(variables.len(), 3);
        assert_eq!(variables.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(variables.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(variables.get("KEY3"), Some(&"value3".to_string()));
        
        // Test list projects
        let projects = db.list_projects()?;
        assert_eq!(projects, vec!["myapp"]);
        
        // Test list workspaces
        let workspaces = db.list_workspaces("myapp")?;
        assert_eq!(workspaces, vec!["config"]);
        
        // Test list varstores
        let varstores = db.list_varstores("myapp", "config")?;
        assert_eq!(varstores, vec!["settings"]);
        
        Ok(())
    }
    
    #[test]
    fn test_find_variables() -> Result<()> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set test variables
        db.set_variable("API_KEY", "secret", &context)?;
        db.set_variable("API_URL", "https://api.example.com", &context)?;
        db.set_variable("DB_PASSWORD", "dbsecret", &context)?;
        
        // Test find with pattern
        let results = db.find_variables("API")?;
        assert_eq!(results.len(), 2);
        
        // Check that results contain expected entries
        let api_key_found = results.iter().any(|(_, _, _, key)| key == "API_KEY");
        let api_url_found = results.iter().any(|(_, _, _, key)| key == "API_URL");
        assert!(api_key_found);
        assert!(api_url_found);
        
        Ok(())
    }
}
