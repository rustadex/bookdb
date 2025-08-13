// src/commands/dec.rs - Decrement command implementation with tests added

use crate::error::{Result, BookdbError};
use crate::ctx::ResolvedContext;
use crate::db::Database;
use stderr::{Stderr, StderrConfig};

/// Execute decrement command: atomically decrement a numeric variable
pub fn execute(
    key: String,
    amount: i64,
    context: &ResolvedContext,
    database: &Database,
) ->  Result<(), E> {
    let mut logger = Stderr::new();
    logger.trace_fn("dec", &format!("decrementing key '{}' by {} in context: {}", key, amount, context));
    
    // Validate inputs
    if key.trim().is_empty() {
        return Err(BookdbError::Argument("Key cannot be empty".to_string()));
    }
    
    // Perform atomic decrement
    match database.decrement_variable(&key, amount, context) {
        Ok(new_value) => {
            // Success - show the new value
            logger.okay(&format!("Decremented '{}' by {}: {}", key, amount, new_value));
            println!("{}", new_value);
            Ok(())
        }
        Err(BookdbError::NonNumericValue(key, value)) => {
            logger.error(&format!("Cannot decrement: Key '{}' has non-numeric value '{}'", key, value));
            logger.info("Hint: Use 'setv' to initialize the key with a numeric value first");
            Err(BookdbError::NonNumericValue(key, value))
        }
        Err(BookdbError::NumericOverflow) => {
            logger.error(&format!("Cannot decrement: Operation would cause underflow"));
            logger.info(&format!("Key '{}' is at minimum value for decrement by {}", key, amount));
            Err(BookdbError::NumericOverflow)
        }
        Err(e) => {
            logger.error(&format!("Failed to decrement '{}': {}", key, e));
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResolvedContext;
    use tempfile::NamedTempFile;
    
    fn create_test_db() -> (Database, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::create_or_open(temp_file.path()).unwrap();
        (db, temp_file)
    }
    
    fn create_test_context() -> ResolvedContext {
        ResolvedContext {
            base_name: "test".to_string(),
            project_name: "TEST_PROJECT".to_string(),
            workspace_name: "TEST_WORKSPACE".to_string(),
            keystore_name: "TEST_KEYSTORE".to_string(),
        }
    }
    
    #[test]
    fn test_decrement_new_key() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Decrement a new key (should initialize to 0 - amount)
        let result = execute("NEW_COUNTER".to_string(), 5, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value
        let value = db.get_variable("NEW_COUNTER", &context).unwrap();
        assert_eq!(value, Some("-5".to_string()));
    }
    
    #[test]
    fn test_decrement_existing_numeric() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set initial value
        db.set_variable("COUNTER", "10", &context).unwrap();
        
        // Decrement by 3
        let result = execute("COUNTER".to_string(), 3, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value
        let value = db.get_variable("COUNTER", &context).unwrap();
        assert_eq!(value, Some("7".to_string()));
    }
    
    #[test]
    fn test_decrement_non_numeric() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set non-numeric value
        db.set_variable("TEXT_KEY", "hello", &context).unwrap();
        
        // Try to decrement - should fail
        let result = execute("TEXT_KEY".to_string(), 1, &context, &db);
        assert!(result.is_err());
        
        if let Err(BookdbError::NonNumericValue(key, value)) = result {
            assert_eq!(key, "TEXT_KEY");
            assert_eq!(value, "hello");
        } else {
            panic!("Expected NonNumericValue error");
        }
    }
    
    #[test]
    fn test_decrement_negative_amount() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set initial value
        db.set_variable("COUNTER", "5", &context).unwrap();
        
        // Decrement by negative amount (effectively increment)
        let result = execute("COUNTER".to_string(), -3, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value
        let value = db.get_variable("COUNTER", &context).unwrap();
        assert_eq!(value, Some("8".to_string()));
    }
    
    #[test]
    fn test_decrement_to_negative() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set initial value
        db.set_variable("COUNTER", "3", &context).unwrap();
        
        // Decrement by more than current value
        let result = execute("COUNTER".to_string(), 10, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value goes negative
        let value = db.get_variable("COUNTER", &context).unwrap();
        assert_eq!(value, Some("-7".to_string()));
    }
    
    #[test]
    fn test_decrement_empty_key() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Try to decrement empty key
        let result = execute("".to_string(), 1, &context, &db);
        assert!(result.is_err());
        
        if let Err(BookdbError::Argument(_)) = result {
            // Expected
        } else {
            panic!("Expected Argument error for empty key");
        }
    }
}
