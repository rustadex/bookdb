// src/commands/ls.rs - Updated with consistent BOOKDB_CONCEPTS.md terminology

use crate::error::{Result, BookdbError};
use crate::bookdb::service::ctx::ResolvedContext;
use crate::bookdb::service::db::Database;
use crate::cli::LsTarget;
use crate::rdx::stderr::{Stderr, StderrConfig};
use std::collections::HashMap;

/// Execute ls command: list items of specified type
pub fn execute(target: LsTarget, context: &ResolvedContext, database: &Database) ->  Result<(), E> {
    let mut logger = Stderr::new();
    logger.trace_fn("ls", &format!("listing {:?} in context: {}", target, context));
    
    match target {
        LsTarget::Keys => list_keys(context, database, &mut logger),
        LsTarget::Projects => list_projects(database, &mut logger),
        LsTarget::Workspaces => list_workspaces(context, database, &mut logger),    // FIXED: was Docstores
        LsTarget::Keystores => list_keystores(context, database, &mut logger),      // FIXED: was Varstores
        LsTarget::Docs => list_docs(context, database, &mut logger),
    }
}

/// List all variables/keys in current context
fn list_keys(context: &ResolvedContext, database: &Database, logger: &mut Stderr) ->  Result<(), E> {
    logger.trace_fn("ls_keys", &format!("listing variables in {}", context));
    
    let variables = database.list_variables(context)?;
    
    if variables.is_empty() {
        logger.info(&format!("No variables found in context: {}", context));
        return Ok(());
    }
    
    // Sort variables by key for consistent output
    let mut var_list: Vec<(&String, &String)> = variables.iter().collect();
    var_list.sort_by_key(|(k, _)| *k);
    
    // Create table data
    let mut table_data = vec![vec!["Key", "Value"]];
    for (key, value) in var_list {
        // Truncate long values for display
        let display_value = if value.len() > 60 {
            format!("{}...", &value[..57])
        } else {
            value.clone()
        };
        table_data.push(vec![key.as_str(), &display_value]);
    }
    
    // Convert to the format expected by simple_table
    let table_refs: Vec<&[&str]> = table_data
        .iter()
        .map(|row| row.as_slice())
        .collect();
    
    // Display with banner
    logger.banner(&format!("Variables in {}", context), '=')?;
    logger.simple_table(&table_refs)?;
    logger.info(&format!("Total: {} variables", variables.len()));
    
    Ok(())
}

/// List all projects in the database
fn list_projects(database: &Database, logger: &mut Stderr) ->  Result<(), E> {
    logger.trace_fn("ls_projects", "listing all projects");
    
    let projects = database.list_projects()?;
    
    if projects.is_empty() {
        logger.info("No projects found");
        return Ok(());
    }
    
    // Create table with numbering
    let mut table_data = vec![vec!["#", "Project"]];
    for (i, project) in projects.iter().enumerate() {
        table_data.push(vec![&(i + 1).to_string(), project]);
    }
    
    let table_refs: Vec<&[&str]> = table_data
        .iter()
        .map(|row| row.as_slice())
        .collect();
    
    logger.banner("Projects", '=')?;
    logger.simple_table(&table_refs)?;
    logger.info(&format!("Total: {} projects", projects.len()));
    
    Ok(())
}

/// List all workspaces in current project
fn list_workspaces(context: &ResolvedContext, database: &Database, logger: &mut Stderr) ->  Result<(), E> {
    logger.trace_fn("ls_workspaces", &format!("listing workspaces in project: {}", context.project));
    
    let workspaces = database.list_workspaces(&context.project)?;
    
    if workspaces.is_empty() {
        logger.info(&format!("No workspaces found in project: {}", context.project));
        return Ok(());
    }
    
    // Create table with numbering
    let mut table_data = vec![vec!["#", "Workspace"]];
    for (i, workspace) in workspaces.iter().enumerate() {
        table_data.push(vec![&(i + 1).to_string(), workspace]);
    }
    
    let table_refs: Vec<&[&str]> = table_data
        .iter()
        .map(|row| row.as_slice())
        .collect();
    
    logger.banner(&format!("Workspaces in {}", context.project), '=')?;
    logger.simple_table(&table_refs)?;
    logger.info(&format!("Total: {} workspaces", workspaces.len()));
    
    Ok(())
}

/// List all keystores in current workspace
fn list_keystores(context: &ResolvedContext, database: &Database, logger: &mut Stderr) ->  Result<(), E> {  // FIXED: was list_varstores
    logger.trace_fn("ls_keystores", &format!("listing keystores in {}.{}", context.project, context.workspace));
    
    let keystores = database.list_keystores(&context.project, &context.workspace)?;  // FIXED: was list_varstores
    
    if keystores.is_empty() {
        logger.info(&format!("No keystores found in {}.{}", context.project, context.workspace));
        return Ok(());
    }
    
    // Create table with numbering and variable counts
    let mut table_data = vec![vec!["#", "Keystore", "Variables"]];  // FIXED: was "Varstore"
    for (i, keystore) in keystores.iter().enumerate() {
        // Create context for this keystore to count variables
        let keystore_context = ResolvedContext {
            base: context.base.clone(),
            project: context.project.clone(),
            workspace: context.workspace.clone(),
            anchor: context.anchor,
            tail: keystore.clone(),
            prefix_mode: context.prefix_mode,
        };
        
        let var_count = database.list_variables(&keystore_context)
            .map(|vars| vars.len())
            .unwrap_or(0);
        
        table_data.push(vec![
            &(i + 1).to_string(), 
            keystore, 
            &var_count.to_string()
        ]);
    }
    
    let table_refs: Vec<&[&str]> = table_data
        .iter()
        .map(|row| row.as_slice())
        .collect();
    
    logger.banner(&format!("Keystores in {}.{}", context.project, context.workspace), '=')?;  // FIXED: was "Varstores"
    logger.simple_table(&table_refs)?;
    logger.info(&format!("Total: {} keystores", keystores.len()));
    
    Ok(())
}

/// List documents in current workspace
fn list_docs(context: &ResolvedContext, _database: &Database, logger: &mut Stderr) ->  Result<(), E> {
    logger.trace_fn("ls_docs", &format!("listing documents in {}.{}", context.project, context.workspace));
    
    // TODO: Implement document listing when document system is ready
    logger.info(&format!("Document listing not yet implemented for {}.{}", context.project, context.workspace));
    logger.info("Use 'bookdb ls keys' to see variables instead");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookdb::service::ctx::{Anchor, ChainMode};
    use tempfile::TempDir;
    
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
    
    fn create_test_db_with_data() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = Database::create_or_open(&db_path).unwrap();
        let context = create_test_context();
        
        // Add some test data
        db.set_variable("TEST_KEY", "test_value", &context).unwrap();
        db.set_variable("ANOTHER_KEY", "another_value", &context).unwrap();
        
        (db, temp_dir)
    }
    
    #[test]
    fn test_list_keys() ->  Result<(), E> {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        let result = execute(LsTarget::Keys, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_workspaces() ->  Result<(), E> {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        let result = execute(LsTarget::Workspaces, &context, &db);  // FIXED: was Docstores
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_keystores() ->  Result<(), E> {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        let result = execute(LsTarget::Keystores, &context, &db);   // FIXED: was Varstores
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_projects() ->  Result<(), E> {
        let (db, _temp) = create_test_db_with_data();
        let context = create_test_context();
        
        let result = execute(LsTarget::Projects, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_empty_context() ->  Result<(), E> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("empty.db");
        let db = Database::create_or_open(&db_path).unwrap();
        let context = create_test_context();
        
        // Test listing keys in empty context
        let result = execute(LsTarget::Keys, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_long_value_truncation() ->  Result<(), E> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = Database::create_or_open(&db_path).unwrap();
        let context = create_test_context();
        
        // Set a very long value
        let long_value = "a".repeat(100);
        db.set_variable("LONG_KEY", &long_value, &context)?;
        
        // List should not fail and should truncate the value
        let result = execute(LsTarget::Keys, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
}
