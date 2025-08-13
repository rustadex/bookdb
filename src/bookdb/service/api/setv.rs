// src/commands/setv.rs - Set variable command implementation
//
// FEATURES:
// 1. Sets variable values in resolved context
// 2. Handles both new variables and updates
// 3. Rich stderr logging with context information
// 4. Proper validation and error handling

use crate::error::{Result, BookdbError};
use crate::bookdb::service::ctx as context::ResolvedContext;
use crate::bookdb::service::db::Database;
use crate::rdx::stderr::{Stderr, StderrConfig};

/// Execute setv command: set variable key=value
pub fn execute(key_value: &str, context: &ResolvedContext, database: &mut Database) ->  Result<(), E> {
    let mut logger = Stderr::new();
    logger.trace_fn("setv", &format!("setting variable: {} in context: {}", key_value, context));
    
    // Parse key=value format
    let (key, value) = key_value.split_once('=')
        .ok_or_else(|| BookdbError::Argument("setv requires key=value format".to_string()))?;
    
    let key = key.trim();
    let value = value.trim();
    
    // Validate key name
    if key.is_empty() {
        return Err(BookdbError::Argument("Key cannot be empty".to_string()));
    }
    
    // Validate key format (no special characters that could break things)
    if key.contains(|c: char| c.is_whitespace() || "=@%#.".contains(c)) {
        return Err(BookdbError::Argument(
            "Key cannot contain whitespace or special characters (=@%#.)".to_string()
        ));
    }
    
    // Check if this is a new variable or update
    let existing_value = database.get_variable(key, context)?;
    
    // Set the variable
    database.set_variable(key, value, context)?;
    
    // Log the operation
    match existing_value {
        None => {
            logger.info(&format!("New variable: '{}' = '{}' in {}", key, value, context));
            logger.trace_fn("setv", &format!("created new variable: {}", key));
        }
        Some(old_value) => {
            if old_value == value {
                logger.trace_fn("setv", &format!("variable {} unchanged (same value)", key));
            } else {
                logger.trace_fn("setv", &format!("updated {} from '{}' to '{}'", key, old_value, value));
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookdb::service::ctx as context::{Anchor, ChainMode};
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
    fn test_setv_new_variable() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set new variable
        execute("API_KEY=secret123", &context, &mut db)?;
        
        // Verify it was set
        let value = db.get_variable("API_KEY", &context)?;
        assert_eq!(value, Some("secret123".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_setv_update_variable() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set initial value
        execute("API_KEY=secret123", &context, &mut db)?;
        
        // Update value
        execute("API_KEY=newsecret456", &context, &mut db)?;
        
        // Verify update
        let value = db.get_variable("API_KEY", &context)?;
        assert_eq!(value, Some("newsecret456".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_setv_with_spaces() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test with spaces around key and value
        execute(" API_KEY = secret with spaces ", &context, &mut db)?;
        
        // Verify trimming worked
        let value = db.get_variable("API_KEY", &context)?;
        assert_eq!(value, Some("secret with spaces".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_setv_invalid_format() {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Missing equals sign
        let result = execute("INVALID_FORMAT", &context, &mut db);
        assert!(result.is_err());
        
        if let Err(BookdbError::Argument(msg)) = result {
            assert!(msg.contains("key=value"));
        } else {
            panic!("Expected argument error for invalid format");
        }
    }
    
    #[test]
    fn test_setv_empty_key() {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        let result = execute("=value", &context, &mut db);
        assert!(result.is_err());
        
        if let Err(BookdbError::Argument(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected argument error for empty key");
        }
    }
    
    #[test]
    fn test_setv_invalid_key_characters() {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test various invalid characters
        let invalid_keys = ["key with space=value", "key@symbol=value", "key.dot=value"];
        
        for invalid_key_value in &invalid_keys {
            let result = execute(invalid_key_value, &context, &mut db);
            assert!(result.is_err(), "Should reject key in: {}", invalid_key_value);
        }
    }
    
    #[test]
    fn test_setv_complex_values() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test complex values
        let test_cases = [
            ("URL", "https://api.example.com/v1/users?limit=100"),
            ("JSON", r#"{"key": "value", "nested": {"data": true}}"#),
            ("MULTILINE", "line1\nline2\nline3"),
            ("SPECIAL_CHARS", "!@#$%^&*()_+-=[]{}|;':\",./<>?"),
        ];
        
        for (key, value) in &test_cases {
            let key_value = format!("{}={}", key, value);
            execute(&key_value, &context, &mut db)?;
            
            let retrieved = db.get_variable(key, &context)?;
            assert_eq!(retrieved, Some(value.to_string()));
        }
        
        Ok(())
    }
}
