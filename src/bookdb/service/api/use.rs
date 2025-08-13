// src/commands/use.rs - Use command implementation
//
// FEATURES:
// 1. Changes active context (cursor) to specified context chain
// 2. Validates context chain syntax
// 3. Updates persistent cursor state
// 4. Rich feedback on context changes

use crate::error::{Result, BookdbError};
use crate::context::{parse_context_chain, ResolvedContext};
use crate::db::Database;
use stderr::{Stderr, StderrConfig};

/// Execute use command: change active context
pub fn execute(
    context_str: &str,
    current_context: &ResolvedContext,
    _database: &Database,
) ->  Result<(), E> {
    let mut logger = Stderr::new();
    logger.trace_fn("use", &format!("changing context from {} to {}", current_context, context_str));
    
    // Parse the new context chain
    let chain = parse_context_chain(context_str, &current_context.base)?;
    
    // Validate the chain
    if chain.project.is_empty() || chain.workspace.is_empty() || chain.tail.is_empty() {
        return Err(BookdbError::InvalidContext(
            "Context chain must have non-empty project, workspace, and tail".to_string()
        ));
    }
    
    // Show context change banner
    logger.banner(&format!("Context Changed"), '=')?;
    logger.info(&format!("Previous: {}", current_context));
    logger.info(&format!("Current:  {}", chain));
    
    // Note: The actual cursor update is handled in main.rs
    // This command just validates and provides feedback
    
    logger.okay("Context updated successfully");
    logger.info("All subsequent commands will use the new context");
    
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
    
    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::create_or_open(&db_path).unwrap();
        (db, temp_dir)
    }
    
    #[test]
    fn test_use_valid_context() ->  Result<(), E> {
        let (db, _temp) = create_test_db();
        let current_context = create_test_context();
        
        let result = execute("@newproject.workspace.var.store", &current_context, &db);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_use_invalid_context() {
        let (db, _temp) = create_test_db();
        let current_context = create_test_context();
        
        // Test invalid context chain
        let result = execute("invalid_context", &current_context, &db);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_use_empty_components() {
        let (db, _temp) = create_test_db();
        let current_context = create_test_context();
        
        // This would be caught by the parser, but test the validation
        // In practice, parse_context_chain would fail first
        let result = execute("@.workspace.var.store", &current_context, &db);
        assert!(result.is_err());
    }
}
