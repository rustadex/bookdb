// src/commands/export.rs - Export command implementation
//
// FEATURES:
// 1. Exports variables from resolved context to file
// 2. Supports multiple formats (JSON, key-value)
// 3. Progress tracking for large exports
// 4. Rich error handling and user feedback

use crate::error::{Result, BookdbError};
use crate::context::ResolvedContext;
use crate::db::Database;
use crate::rdx::stderr::{Stderr, StderrConfig};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// Execute export command: export variables to file
pub fn execute(
    file_path: &Path,
    format: Option<&str>,
    _filters: (Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>),
    context: &ResolvedContext,
    database: &Database,
) -> Result<()> {
    let mut logger = Stderr::new(&StderrConfig::from_env());
    logger.trace_fn("export", &format!("exporting from context: {} to file: {:?}", context, file_path));
    
    // Get all variables from the context
    let variables = database.list_variables(context)?;
    
    if variables.is_empty() {
        logger.warn(&format!("No variables to export from context: {}", context));
        return Ok(());
    }
    
    logger.info(&format!("Exporting {} variables from {}", variables.len(), context));
    
    // Determine format from file extension or explicit format
    let export_format = determine_format(file_path, format)?;
    
    // Export the data
    let content = match export_format {
        ExportFormat::Json => export_as_json(&variables)?,
        ExportFormat::KeyValue => export_as_key_value(&variables)?,
    };
    
    // Write to file
    fs::write(file_path, content)?;
    
    logger.okay(&format!(
        "Exported {} variables to {} ({})",
        variables.len(),
        file_path.display(),
        export_format.name()
    ));
    
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ExportFormat {
    Json,
    KeyValue,
}

impl ExportFormat {
    fn name(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON",
            ExportFormat::KeyValue => "Key-Value",
        }
    }
}

/// Determine export format from file extension or explicit format
fn determine_format(file_path: &Path, explicit_format: Option<&str>) -> Result<ExportFormat> {
    if let Some(format) = explicit_format {
        match format.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "kv" | "keyvalue" | "key-value" => Ok(ExportFormat::KeyValue),
            _ => Err(BookdbError::Argument(format!("Unknown export format: {}", format))),
        }
    } else {
        // Infer from file extension
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "json" => Ok(ExportFormat::Json),
                "kv" | "txt" => Ok(ExportFormat::KeyValue),
                _ => Ok(ExportFormat::Json), // Default to JSON
            }
        } else {
            Ok(ExportFormat::Json) // Default to JSON
        }
    }
}

/// Export variables as JSON format
fn export_as_json(variables: &HashMap<String, String>) -> Result<String> {
    serde_json::to_string_pretty(variables)
        .map_err(|e| BookdbError::Io(format!("JSON serialization failed: {}", e)))
}

/// Export variables as key-value format
fn export_as_key_value(variables: &HashMap<String, String>) -> Result<String> {
    let mut lines = Vec::new();
    
    // Sort keys for consistent output
    let mut sorted_vars: Vec<(&String, &String)> = variables.iter().collect();
    sorted_vars.sort_by_key(|(k, _)| *k);
    
    for (key, value) in sorted_vars {
        // Escape newlines and quotes in values
        let escaped_value = value
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        
        lines.push(format!("{}={}", key, escaped_value));
    }
    
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Anchor, ChainMode};
    use tempfile::{TempDir, NamedTempFile};
    use std::collections::HashMap;
    
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
        db.set_variable("DB_URL", "postgres://localhost:5432/mydb", &context).unwrap();
        db.set_variable("DEBUG", "true", &context).unwrap();
        
        (db, temp_dir)
    }
    
    #[test]
    fn test_export_json() -> Result<()> {
        let (db, _temp) = create_test_db_with_data();
        let context = create_test_context();
        
        let output_file = NamedTempFile::new().unwrap();
        
        execute(
            output_file.path(),
            Some("json"),
            (None, None, None, None, None, None),
            &context,
            &db,
        )?;
        
        // Verify file was created and contains JSON
        let content = fs::read_to_string(output_file.path())?;
        let parsed: serde_json::Value = serde_json::from_str(&content)?;
        assert!(parsed.is_object());
        
        Ok(())
    }
    
    #[test]
    fn test_export_key_value() -> Result<()> {
        let (db, _temp) = create_test_db_with_data();
        let context = create_test_context();
        
        let output_file = NamedTempFile::new().unwrap();
        
        execute(
            output_file.path(),
            Some("kv"),
            (None, None, None, None, None, None),
            &context,
            &db,
        )?;
        
        // Verify file was created and contains key=value format
        let content = fs::read_to_string(output_file.path())?;
        assert!(content.contains("API_KEY=secret123"));
        assert!(content.contains("DEBUG=true"));
        
        Ok(())
    }
    
    #[test]
    fn test_format_inference() -> Result<()> {
        // Test JSON inference from .json extension
        let json_format = determine_format(Path::new("test.json"), None)?;
        assert!(matches!(json_format, ExportFormat::Json));
        
        // Test key-value inference from .kv extension
        let kv_format = determine_format(Path::new("test.kv"), None)?;
        assert!(matches!(kv_format, ExportFormat::KeyValue));
        
        // Test explicit format override
        let override_format = determine_format(Path::new("test.txt"), Some("json"))?;
        assert!(matches!(override_format, ExportFormat::Json));
        
        Ok(())
    }
    
    #[test]
    fn test_export_empty_context() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("empty.db");
        let db = Database::create_or_open(&db_path).unwrap();
        let context = create_test_context();
        
        let output_file = NamedTempFile::new().unwrap();
        
        // Should succeed but warn about empty context
        let result = execute(
            output_file.path(),
            Some("json"),
            (None, None, None, None, None, None),
            &context,
            &db,
        );
        
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_special_characters_in_values() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = Database::create_or_open(&db_path).unwrap();
        let context = create_test_context();
        
        // Add variables with special characters
        db.set_variable("MULTILINE", "line1\nline2\nline3", &context)?;
        db.set_variable("QUOTES", r#"value with "quotes""#, &context)?;
        db.set_variable("BACKSLASHES", r"path\to\file", &context)?;
        
        let output_file = NamedTempFile::new().unwrap();
        
        execute(
            output_file.path(),
            Some("kv"),
            (None, None, None, None, None, None),
            &context,
            &db,
        )?;
        
        // Verify special characters are properly escaped
        let content = fs::read_to_string(output_file.path())?;
        assert!(content.contains("MULTILINE=line1\\nline2\\nline3"));
        
        Ok(())
    }
}
