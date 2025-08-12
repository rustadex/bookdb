// src/db/project.rs - Project-level database operations

use crate::error::Result;
use crate::sql;
use rusqlite::{params, Transaction};
use super::Database;

impl Database {
    /// List all projects in the database
    pub fn list_projects(&self) -> Result<Vec<String>> {
        self.logger.trace_fn("database", "listing all projects");
        
        let mut stmt = self.connection.prepare(sql::LIST_PROJECTS)?;
        let project_iter = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }
        
        Ok(projects)
    }
    
    /// Ensure a project exists in the database
    pub fn ensure_project_exists(&self, project: &str) -> Result<()> {
        self.logger.trace_fn("database", &format!("ensuring project exists: {}", project));
        
        self.connection.execute(sql::CREATE_PROJECT, params![project])?;
        Ok(())
    }
    
    /// Ensure project exists within transaction, return project ID
    pub fn ensure_project_exists_tx(&self, tx: &Transaction, project: &str) -> Result<i64> {
        let mut stmt = tx.prepare(sql::RESOLVE_PROJECT_ID)?;
        let mut rows = stmt.query_map([project], |row| {
            Ok(row.get::<_, i64>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(project_id)) => Ok(project_id),
            Some(Err(e)) => Err(e.into()),
            None => {
                // Create the project
                tx.execute(sql::CREATE_PROJECT, params![project])?;
                Ok(tx.last_insert_rowid())
            }
        }
    }
    
    /// Create a new project
    pub fn create_project(&self, name: &str) -> Result<()> {
        self.logger.trace_fn("database", &format!("creating project: {}", name));
        
        self.connection.execute(sql::CREATE_PROJECT, params![name])?;
        Ok(())
    }
    
    /// Delete a project (stub - not implemented)
    pub fn delete_project(&self, _name: &str) -> Result<()> {
        // TODO: Implement project deletion with cascade
        todo!("Project deletion not yet implemented")
    }
}
