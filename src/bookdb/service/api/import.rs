// src/commands/import.rs - Import command implementation
//
// FEATURES:
// 1. Imports variables from file to resolved context
// 2. Supports multiple formats (JSON, key-value)
// 3. Interactive confirmation for overwrites
// 4. Progress tracking and rich feedback

use crate::error::{Result, BookdbError};
use crate::bookdb::service::ctx::ResolvedContext;
use crate::bookdb::service::db::Database;
use crate::rdx::stderr::{Stderr, StderrConfig};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// Execute import command: import variables from file
pub fn execute(
    file_path: &Path,
    format: Option<&str>,
    _mappings: (Option<&str>, Option<&str>, Option<&str>),
    context: &ResolvedContext,
    database: &mut Database,
) ->  Result<(), E> {
    let mut logger = Stderr::new();
    logger.trace_fn("import", &format!("importing from file: {:?} to context: {}", file_path, context));
    
    // Check if file exists
    if !file_path.exists() {
        return Err(BookdbError::Io(format!("File not found: {:?}", file_path)));
    }
    
    // Read file content
    let content = fs::read_to_string(file_path)
        .map_err(|e| BookdbError::Io(format!("Failed to read file: {}", e)))?;
    
    // Determine format and parse variables
    let import_format = determine_format(file_path, format)?;
    let variables = match import_format {
        ImportFormat::Json => parse_json(&content)?,
        ImportFormat::KeyValue => parse_key_value(&content)?,
    };
    
    if variables.is_empty() {
        logger.warn("No variables found in import file");
        return Ok(());
    }
    
    logger.info(&format!("Found {} variables to import", variables.len()));
    
    // Check for existing variables that would be overwritten
    let existing_vars = database.list_variables(context)?;
    let conflicts: Vec<&String> = variables.keys()
        .filter(|key| existing_vars.contains_key(*key))
        .collect();
    
    if !conflicts.is_empty() {
        logger.warn(&format!("{} variables will be overwritten:", conflicts.len()));
        for key in &conflicts {
            logger.warn(&format!("  - {}", key));
        }
        
        // Ask for confirmation
        let confirmation_msg = format!(
            "Import {} variables to context '{}'? This will overwrite {} existing variables.",
            variables.len(),
            context,
            conflicts.len()
        );
        
        if let Ok(Some(confirmed)) = logger.confirm(&confirmation_msg) {
            if !confirmed {
                logger.info("Import cancelled");
                return Ok(());
            }
        } else {
            return Err(BookdbError::Argument("Confirmation required. Use --yes flag or run interactively.".to_string()));
        }
    }
    
    // Import variables
    logger.trace_fn("import", "starting variable import");
    database.import_variables(variables.clone(), context)?;
    
    logger.okay(&format!(
        "Imported {} variables from {} ({})",
        variables.len(),
        file_path.display(),
        import_format.name()
    ));
    
    // Show summary of what was imported
    if variables.len() <= 10 {
        logger.info("Imported variables:");
        for key in variables.keys() {
            logger.info(&format!("  - {}", key));
        }
    } else {
        logger.info(&format!("Imported {} variables (too many to list)", variables.len()));
    }
    
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ImportFormat {
    Json,
    KeyValue,
}

impl ImportFormat {
    fn name(&self) -> &'static str {
        match self {
            ImportFormat::Json => "JSON",
            ImportFormat::KeyValue => "Key-Value",
        }
    }
}

/// Determine import format from file extension or explicit format
fn determine_format(file_path: &Path, explicit_format: Option<&str>) -> Result<ImportFormat> {
    if let Some(format) = explicit_format {
        match format.to_lowercase().as_str() {
            "json" => Ok(ImportFormat::Json),
            "kv" | "keyvalue" | "key-value" => Ok(ImportFormat::KeyValue),
            _ => Err(BookdbError::Argument(format!("Unknown import format: {}", format))),
        }
    } else {
        // Infer from file extension
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "json" => Ok(ImportFormat::Json),
                "kv" | "txt" => Ok(ImportFormat::KeyValue),
                _ => {
                    // Try to auto-detect from content
                    let content = fs::read_to_string(file_path)
                        .map_err(|e| BookdbError::Io(format!("Failed to read file for format detection: {}", e)))?;
                    
                    if content.trim_start().starts_with('{') {
                        Ok(ImportFormat::Json)
                    } else {
                        Ok(ImportFormat::KeyValue)
                    }
                }
            }
        } else {
            Ok(ImportFormat::Json) // Default to JSON
        }
    }
}

/// Parse JSON format variables
fn parse_json(content: &str) -> Result<HashMap<String, String>> {
    let parsed: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| BookdbError::Parse(format!("Invalid JSON: {}", e)))?;
    
    let mut variables = HashMap::new();
    
    if let serde_json::Value::Object(obj) = parsed {
        for (key, value) in obj {
            let string_value = match value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "".to_string(),
                _ => serde_json::to_string(&value)
                    .map_err(|e| BookdbError::Parse(format!("Failed to serialize value for key {}: {}", key, e)))?,
            };
            variables.insert(key, string_value);
        }
    } else {
        return Err(BookdbError::Parse("JSON root must be an object".to_string()));
    }
    
    Ok(variables)
}

/// Parse key-value format variables
fn parse_key_value(content: &str) -> Result<HashMap<String, String>> {
    let mut variables = HashMap::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            if key.is_empty() {
                return Err(BookdbError::Parse(format!("Empty key on line {}", line_num + 1)));
            }
            
            // Unescape common escape sequences
            let unescaped_value = value
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .replace("\\\\", "\\");
            
            variables.insert(key.to_string(), unescaped_value);
        } else {
            return Err(BookdbError::Parse(format!("Invalid key=value format on line {}: {}", line_num + 1, line)));
        }
    }
    
    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookdb::service::ctx as context::{Anchor, ChainMode};
    use tempfile::{TempDir, NamedTempFile};
    use std::io::Write;
    
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
    fn test_import_json() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Create JSON file
        let mut json_file = NamedTempFile::new().unwrap();
        writeln!(json_file, r#"{{
            "API_KEY": "secret123",
            "DB_URL": "postgres://localhost:5432/mydb",
            "DEBUG": "true",
            "PORT": 3000
        }}"#)?;
        
        execute(
            json_file.path(),
            Some("json"),
            (None, None, None),
            &context,
            &mut db,
        )?;
        
        // Verify variables were imported
        assert_eq!(db.get_variable("API_KEY", &context)?, Some("secret123".to_string()));
        assert_eq!(db.get_variable("DEBUG", &context)?, Some("true".to_string()));
        assert_eq!(db.get_variable("PORT", &context)?, Some("3000".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_import_key_value() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Create key-value file
        let mut kv_file = NamedTempFile::new().unwrap();
        writeln!(kv_file, "API_KEY=secret123")?;
        writeln!(kv_file, "DB_URL=postgres://localhost:5432/mydb")?;
        writeln!(kv_file, "DEBUG=true")?;
        writeln!(kv_file, "# This is a comment")?;
        writeln!(kv_file, "")?;
        writeln!(kv_file, "MULTILINE=line1\\nline2\\nline3")?;
        
        execute(
            kv_file.path(),
            Some("kv"),
            (None, None, None),
            &context,
            &mut db,
        )?;
        
        // Verify variables were imported
        assert_eq!(db.get_variable("API_KEY", &context)?, Some("secret123".to_string()));
        assert_eq!(db.get_variable("DEBUG", &context)?, Some("true".to_string()));
        assert_eq!(db.get_variable("MULTILINE", &context)?, Some("line1\nline2\nline3".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_format_auto_detection() ->  Result<(), E> {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Test JSON auto-detection
        let mut json_file = NamedTempFile::with_suffix(".unknown").unwrap();
        writeln!(json_file, r#"{{"KEY": "value"}}"#)?;
        
        let json_format = determine_format(json_file.path(), None)?;
        assert!(matches!(json_format, ImportFormat::Json));
        
        // Test key-value auto-detection
        let mut kv_file = NamedTempFile::with_suffix(".unknown").unwrap();
        writeln!(kv_file, "KEY=value")?;
        
        let kv_format = determine_format(kv_file.path(), None)?;
        assert!(matches!(kv_format, ImportFormat::KeyValue));
        
        Ok(())
    }
    
    #[test]
    fn test_import_nonexistent_file() {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        let result = execute(
            Path::new("/nonexistent/file.json"),
            None,
            (None, None, None),
            &context,
            &mut db,
        );
        
        assert!(result.is_err());
        if let Err(BookdbError::Io(msg)) = result {
            assert!(msg.contains("File not found"));
        } else {
            panic!("Expected IO error for nonexistent file");
        }
    }
    
    #[test]
    fn test_import_invalid_json() {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Create invalid JSON file
        let mut json_file = NamedTempFile::new().unwrap();
        writeln!(json_file, "{{invalid json")?;
        
        let result = execute(
            json_file.path(),
            Some("json"),
            (None, None, None),
            &context,
            &mut db,
        );
        
        assert!(result.is_err());
        if let Err(BookdbError::Parse(msg)) = result {
            assert!(msg.contains("Invalid JSON"));
        } else {
            panic!("Expected parse error for invalid JSON");
        }
    }
    
    #[test]
    fn test_import_invalid_key_value() {
        let (mut db, _temp) = create_test_db();
        let context = create_test_context();
        
        // Create invalid key-value file
        let mut kv_file = NamedTempFile::new().unwrap();
        writeln!(kv_file, "VALID=value")?;
        writeln!(kv_file, "invalid line without equals")?;
        
        let result = execute(
            kv_file.path(),
            Some("kv"),
            (None, None, None),
            &context,
            &mut db,
        );
        
        assert!(result.is_err());
        if let Err(BookdbError::Parse(msg)) = result {
            assert!(msg.contains("Invalid key=value format"));
        } else {
            panic!("Expected parse error for invalid key-value format");
        }
    }
}
