// src/db/core.rs - Core database functionality

use std::path::Path;
use stderr::{Stderr, StderrConfig};
use rusqlite::{Connection, Transaction};

use crate::error::{Result, BookdbError};

use crate::sql;



/// Core database connection and schema management
pub struct Database {
    pub connection: Connection,
    pub logger: Stderr,
    pub base_name: String,
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
        self.logger.trace_fn("database", "setting up schema from external files");
        
        // Use external SQL files via sql.rs
        self.connection.execute(sql::V1_CREATE_TABLES, [])?;
        self.connection.execute(sql::V2_CREATE_DOCS, [])?;
        
        self.logger.trace_fn("database", "schema setup complete");
        Ok(())
    }
    
    /// Execute raw SQL (for installation and setup)
    pub fn execute_sql(&self, sql: &str) ->  Result<(), E> {
        self.logger.trace_fn("database", "executing raw SQL");
        self.connection.execute(sql, [])?;
        Ok(())
    }
    
    /// Begin a transaction
    pub fn transaction(&self) -> Result<Transaction> {
        Ok(self.connection.transaction()?)
    }
}
