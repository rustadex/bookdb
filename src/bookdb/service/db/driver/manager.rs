// src/bookdb/service/db/driver/manager.rs - Multi-base database manager

use crate::bookdb::app::sup::error::Result;
use super::core::Database;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DatabaseManager {
    pub databases: HashMap<String, Database>,
    pub active_base: Option<String>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            databases: HashMap::new(),
            active_base: None,
        }
    }

    pub fn add_base(&mut self, name: String, path: PathBuf) -> Result<()> {
        let db = Database::connect(&path)?;
        self.databases.insert(name, db);
        Ok(())
    }

    pub fn get_active(&self) -> Option<&Database> {
        if let Some(ref active) = self.active_base {
            self.databases.get(active)
        } else {
            None
        }
    }

    pub fn set_active(&mut self, name: String) -> Result<()> {
        if self.databases.contains_key(&name) {
            self.active_base = Some(name);
            Ok(())
        } else {
            Err("Base not found".into())
        }
    }
}
