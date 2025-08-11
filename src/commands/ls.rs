// src/commands/ls.rs - List command implementation
//
// FEATURES:
// 1. Lists various types: keys, projects, workspaces, varstores
// 2. Rich table formatting with stderr integration
// 3. Hierarchical display with context awareness
// 4. Comprehensive error handling

use crate::error::{Result, BookdbError};
use crate::context::ResolvedContext;
use crate::db::Database;
use crate::cli::LsTarget;
use crate::rdx::stderr::{Stderr, StderrConfig};
use std::collections::HashMap;

/// Execute ls command: list items of specified type
pub fn execute(target: LsTarget, context: &ResolvedContext, database: &Database) -> Result<()> {
    let mut logger = Stderr::new(&StderrConfig::from_env());
    logger.trace_fn("ls", &format!("listing {:?} in context: {}", target, context));
    
    match target {
        LsTarget::Keys => list_keys(context, database, &mut logger),
        LsTarget::Projects => list_projects(database, &mut logger),
        LsTarget::Docstores => list_workspaces(context, database, &mut logger),
        LsTarget::Varstores => list_varstores(context, database, &mut logger),
        LsTarget::Docs => list_docs(context, database, &mut logger),
    }
}

/// List all variables/keys in current context
fn list_keys(context: &ResolvedContext, database: &Database, logger: &mut Stderr) -> Result<()> {
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
fn list_projects(database: &Database, logger: &mut Stderr) -> Result<()> {
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
fn list_workspaces(context: &ResolvedContext, database: &Database, logger: &mut Stderr) -> Result<()> {
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

/// List all varstores in current workspace
fn list_varstores(context: &ResolvedContext, database: &Database, logger: &mut Stderr) -> Result<()> {
    logger.trace_fn("ls_varstores", &format!("listing varstores in {}.{}", context.project, context.workspace));
    
    let varstores = database.list_varstores(&context.project, &context.workspace)?;
    
    if varstores.is_empty() {
        logger.info(&format!("No varstores found in {}.{}", context.project, context.workspace));
        return Ok(());
    }
    
    // Create table with numbering and variable counts
    let mut table_data = vec![vec!["#", "Varstore", "Variables"]];
    for (i, varstore) in varstores.iter().enumerate() {
        // Create context for this varstore to count variables
        let varstore_context = ResolvedContext {
            base: context.base.clone(),
            project: context.project.clone(),
            workspace: context.workspace.clone(),
            anchor: context.anchor,
            tail: varstore.clone(),
            prefix_mode: context.prefix_mode,
        };
        
        let var_count = database.list_variables(&varstore_context)
            .map(|vars| vars.len())
            .unwrap_or(0);
        
        table_data.push(vec![
            &(i + 1).to_string(), 
            varstore, 
            &var_count.to_string()
        ]);
    }
    
    let table_refs: Vec<&[&str]> = table_data
        .iter()
        .map(|row| row.as_slice())
        .collect();
    
    logger.banner(&format!("Varstores in {}.{}", context.project, context.workspace), '=')?;
    logger.simple_table(&table_refs)?;
    logger.info(&format!("Total: {} varstores", varstores.len()));
    
    Ok(())
}

/// List documents (placeholder for future implementation)
fn list_docs(context: &ResolvedContext, _database: &Database, logger: &mut Stderr) -> Result<()> {
    logger.trace_fn("ls_docs", &format!("listing documents in {}", context));
    
    // For now, document support is not fully implemented
    logger.info("Document listing not yet implemented");
    logger.info("Use 'bookdb ls keys' to see variables in current context");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Anchor, ChainMode};
    use tempfile::TempDir;
    
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
    
    fn create_test_db_with_data() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = Database::create_or_open(&db_path).unwrap();
        
        let context = create_test_context();
        
        // Add test data
        db.set_variable("API_KEY", "secret123", &context).unwrap();
        db.set_variable("DB_URL", "postgres://localhost", &context).unwrap();
        db.set_variable("DEBUG", "true", &context).unwrap();
        
        // Add another varstore
        let context2 = ResolvedContext {
            tail: "production".to_string(),
            ..context
        };
        db.set_variable("PROD_KEY", "prod_value", &context2).unwrap();
        
        (db, temp_dir)
    }
    
    #[test]
    fn test_list_keys() -> Result<()> {
        let (db, _temp) = create_test_db_with_data();
        let context = create_test_context();
        
        let result = execute(LsTarget::Keys, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_projects() -> Result<()> {
        let (db, _temp) = create_test_db_with_data();
        let context = create_test_context();
        
        let result = execute(LsTarget::Projects, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_varstores() -> Result<()> {
        let (db, _temp) = create_test_db_with_data();
        let context = create_test_context();
        
        let result = execute(LsTarget::Varstores, &context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_list_empty_context() -> Result<()> {
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
    fn test_long_value_truncation() -> Result<()> {
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
