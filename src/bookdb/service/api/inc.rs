// src/commands/inc.rs - Increment command implementation with tests added

use crate::error::{Result, BookdbError};
use crate::bookdb::service::ctx as context::ResolvedContext;
use crate::bookdb::service::db::Database;
use crate::rdx::stderr::{Stderr, StderrConfig};

/// Execute increment command: atomically increment a numeric variable
pub fn execute(
    key: String,
    amount: i64,
    context: &ResolvedContext,
    database: &Database,
) ->  Result<(), E> {
    let mut logger = Stderr::new();
    logger.trace_fn("inc", &format!("incrementing key '{}' by {} in context: {}", key, amount, context));
    
    // Validate inputs
    if key.trim().is_empty() {
        return Err(BookdbError::Argument("Key cannot be empty".to_string()));
    }
    
    // Perform atomic increment
    match database.increment_variable(&key, amount, context) {
        Ok(new_value) => {
            // Success - show the new value
            logger.okay(&format!("Incremented '{}' by {}: {}", key, amount, new_value));
            println!("{}", new_value);
            Ok(())
        }
        Err(BookdbError::NonNumericValue(key, value)) => {
            logger.error(&format!("Cannot increment: Key '{}' has non-numeric value '{}'", key, value));
            logger.info("Hint: Use 'setv' to initialize the key with a numeric value first");
            Err(BookdbError::NonNumericValue(key, value))
        }
        Err(BookdbError::NumericOverflow) => {
            logger.error(&format!("Cannot increment: Operation would cause overflow"));
            logger.info(&format!("Key '{}' is at maximum value for increment by {}", key, amount));
            Err(BookdbError::NumericOverflow)
        }
        Err(e) => {
            logger.error(&format!("Failed to increment '{}': {}", key, e));
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookdb::service::ctx as context::ResolvedContext;
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
    fn test_increment_negative_amount() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set initial value
        db.set_variable("COUNTER", "10", &context).unwrap();
        
        // Increment by negative amount (effectively decrement)
        let result = execute("COUNTER".to_string(), -3, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value
        let value = db.get_variable("COUNTER", &context).unwrap();
        assert_eq!(value, Some("7".to_string()));
    }
    
    #[test]
    fn test_increment_empty_key() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Try to increment empty key
        let result = execute("".to_string(), 1, &context, &db);
        assert!(result.is_err());
        
        if let Err(BookdbError::Argument(_)) = result {
            // Expected
        } else {
            panic!("Expected Argument error for empty key");
        }
    }
}[test]
    fn test_increment_new_key() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Increment a new key (should initialize to amount)
        let result = execute("NEW_COUNTER".to_string(), 5, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value
        let value = db.get_variable("NEW_COUNTER", &context).unwrap();
        assert_eq!(value, Some("5".to_string()));
    }
    
    #[test]
    fn test_increment_existing_numeric() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set initial value
        db.set_variable("COUNTER", "10", &context).unwrap();
        
        // Increment by 3
        let result = execute("COUNTER".to_string(), 3, &context, &db);
        assert!(result.is_ok());
        
        // Verify the value
        let value = db.get_variable("COUNTER", &context).unwrap();
        assert_eq!(value, Some("13".to_string()));
    }
    
    #[test]
    fn test_increment_non_numeric() {
        let (db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Set non-numeric value
        db.set_variable("TEXT_KEY", "hello", &context).unwrap();
        
        // Try to increment - should fail
        let result = execute("TEXT_KEY".to_string(), 1, &context, &db);
        assert!(result.is_err());
        
        if let Err(BookdbError::NonNumericValue(key, value)) = result {
            assert_eq!(key, "TEXT_KEY");
            assert_eq!(value, "hello");
        } else {
            panic!("Expected NonNumericValue error");
        }
    }
    
    #
