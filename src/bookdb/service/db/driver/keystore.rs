// src/db/keystore.rs - Keystore (variable) database operations

use crate::error::{Result, BookdbError};
use crate::context::ResolvedContext;
use crate::sql;
use rusqlite::{params, Transaction};
use std::collections::HashMap;
use super::Database;

impl Database {
    /// List all keystores in a workspace within a project  
    pub fn list_keystores(&self, project: &str, workspace: &str) -> Result<Vec<String>> {
        self.logger.trace_fn("database", &format!("listing keystores in {}.{}", project, workspace));
        
        let mut stmt = self.connection.prepare(sql::LIST_KEYSTORES)?;
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
        
        let mut stmt = self.connection.prepare(sql::LIST_VARIABLES)?;
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
    
    /// Get variable value by key and context
    pub fn get_variable(&self, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        self.logger.trace_fn("database", &format!("getting variable {} in context: {}", key, context));
        
        let mut stmt = self.connection.prepare(sql::GET_VARIABLE)?;
        let mut rows = stmt.query_map([&context.project, &context.workspace, &context.tail, key], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
    
    /// Set variable value (upsert operation)
    pub fn set_variable(&self, key: &str, value: &str, context: &ResolvedContext) ->  Result<(), E> {
        self.logger.trace_fn("database", &format!("setting variable {} = {} in context: {}", key, value, context));
        
        let tx = self.connection.transaction()?;
        
        // Ensure context exists
        let kvns_id = self.ensure_keystore_context_exists(&tx, context)?;
        
        // Set the variable
        tx.execute(sql::SET_VARIABLE, params![key, value, kvns_id])?;
        
        tx.commit()?;
        Ok(())
    }
    
    /// Delete variable by key and context
    pub fn delete_variable(&self, key: &str, context: &ResolvedContext) -> Result<bool, E> {
        self.logger.trace_fn("database", &format!("deleting variable {} in context: {}", key, context));
        
        let changes = self.connection.execute(
            sql::DELETE_VARIABLE,
            params![&context.project, &context.workspace, &context.tail, key]
        )?;
        
        Ok(changes > 0)
    }
    
    /// Atomically increment a numeric variable
    pub fn increment_variable(&self, key: &str, amount: i64, context: &ResolvedContext) -> Result<i64> {
        self.logger.trace_fn("database", &format!("incrementing variable {} by {} in context: {}", key, amount, context));
        
        let tx = self.connection.transaction()?;
        
        // Ensure context exists
        let kvns_id = self.ensure_keystore_context_exists(&tx, context)?;
        
        // Get current value
        let current_value = match self.get_variable_in_tx(&tx, key, context)? {
            Some(val) => {
                val.parse::<i64>()
                   .map_err(|_| BookdbError::NonNumericValue(key.to_string(), val))?
            }
            None => 0, // Initialize to 0 if key doesn't exist
        };
        
        // Calculate new value with overflow check
        let new_value = current_value
            .checked_add(amount)
            .ok_or_else(|| BookdbError::NumericOverflow(key.to_string(), current_value, amount))?;
        
        // Update the variable
        tx.execute(sql::SET_VARIABLE, params![key, new_value.to_string(), kvns_id])?;
        
        tx.commit()?;
        Ok(new_value)
    }
    
    /// Atomically decrement a numeric variable
    pub fn decrement_variable(&self, key: &str, amount: i64, context: &ResolvedContext) -> Result<i64> {
        self.logger.trace_fn("database", &format!("decrementing variable {} by {} in context: {}", key, amount, context));
        
        // Use increment with negative amount
        self.increment_variable(key, -amount, context)
    }
    
    /// Count variables in a context
    pub fn count_variables(&self, context: &ResolvedContext) -> Result<usize> {
        self.logger.trace_fn("database", &format!("counting variables in context: {}", context));
        
        let count: i64 = self.connection.query_row(
            sql::COUNT_VARIABLES,
            params![&context.project, &context.workspace, &context.tail],
            |row| Ok(row.get(0)?)
        )?;
        
        Ok(count as usize)
    }
    
    /// Get variable value within a transaction
    fn get_variable_in_tx(&self, tx: &Transaction, key: &str, context: &ResolvedContext) -> Result<Option<String>> {
        let mut stmt = tx.prepare(sql::GET_VARIABLE)?;
        let mut rows = stmt.query_map([&context.project, &context.workspace, &context.tail, key], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
    
    /// Ensure keystore context (project.workspace.keystore) exists, return keyval_ns ID
    fn ensure_keystore_context_exists(&self, tx: &Transaction, context: &ResolvedContext) -> Result<i64> {
        // First ensure project exists
        let project_id = self.ensure_project_exists_tx(tx, &context.project)?;
        
        // Then ensure keyval namespace exists
        let mut stmt = tx.prepare(sql::RESOLVE_KEYVAL_NS_ID)?;
        let mut rows = stmt.query_map([&context.project, &context.workspace, &context.tail], |row| {
            Ok(row.get::<_, i64>(0)?)
        })?;
        
        match rows.next() {
            Some(Ok(kvns_id)) => Ok(kvns_id),
            Some(Err(e)) => Err(e.into()),
            None => {
                // Create the keyval namespace
                tx.execute(sql::CREATE_KEYVAL_NS, params![&context.tail, project_id, &context.workspace])?;
                Ok(tx.last_insert_rowid())
            }
        }
    }
    
    /// Create a new keystore (stub - implicit through first variable)
    pub fn create_keystore(&self, _project: &str, _workspace: &str, _name: &str) ->  Result<(), E> {
        // TODO: Implement explicit keystore creation if needed
        todo!("Explicit keystore creation not yet implemented")
    }
    
    /// Delete a keystore (stub - not implemented)
    pub fn delete_keystore(&self, _project: &str, _workspace: &str, _name: &str) ->  Result<(), E> {
        // TODO: Implement keystore deletion with cascade
        todo!("Keystore deletion not yet implemented")
    }
}
