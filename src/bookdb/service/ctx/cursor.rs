


use crate::error::{Result, BookdbError};
use crate::config::Config;
use super::typesV1::{CursorState, ContextChain};
use serde::{Serialize, Deserialize};
use std::fs;

// ============================================================================
// CURSOR STATE OPERATIONS
// ============================================================================



impl CursorState {
    /// Load cursor state from disk
    pub fn load_from_disk(config: &Config) -> Result<Self> {
        let cursor_file = config.get_cursor_file_path();
        
        if !cursor_file.exists() {
            return Ok(CursorState::default());
        }
        
        let content = fs::read_to_string(&cursor_file)
            .map_err(|e| BookdbError::Io(format!("Failed to read cursor file: {}", e)))?;
            
        let cursor_state: CursorState = serde_json::from_str(&content)
            .map_err(|e| BookdbError::ConfigParse(format!("Invalid cursor file: {}", e)))?;
        
        Ok(cursor_state)
    }
    
    /// Save cursor state to disk
    pub fn save_to_disk(&self, config: &Config) -> Result<()> {
        let cursor_file = config.get_cursor_file_path();
        
        // Ensure parent directory exists
        if let Some(parent) = cursor_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| BookdbError::Io(format!("Failed to create cursor directory: {}", e)))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| BookdbError::ConfigParse(format!("Failed to serialize cursor state: {}", e)))?;
            
        fs::write(&cursor_file, content)
            .map_err(|e| BookdbError::Io(format!("Failed to write cursor file: {}", e)))?;
        
        Ok(())
    }
    
    /// Update the context cursor
    pub fn update_context(&mut self, new_context: &ContextChain) {
        // Update base cursor if explicitly specified
        if let Some(ref base) = new_context.base {
            self.base_cursor = base.clone();
        }
        
        // Update context cursor
        self.context_cursor = Some(new_context.clone());
    }
    
    /// Get the current context or return the invincible superchain as fallback
    pub fn get_current_context(&self) -> ContextChain {
        if let Some(ref context) = self.context_cursor {
            context.clone()
        } else {
            // Use DefaultResolver to create invincible superchain
            use super::resolver::DefaultResolver;
            DefaultResolver::create_invincible_superchain(&self.base_cursor)
        }
    }
}

// ============================================================================
// LEGACY COMPATIBILITY (for existing simple cursor operations)
// ============================================================================

use crate::config::Paths;

/// Legacy cursor reading (for backward compatibility)
pub fn read_cursor(paths: &Paths) -> (Option<String>, Option<String>) {
    let base = fs::read_to_string(&paths.cursor_base_path).ok()
        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let chain = fs::read_to_string(&paths.cursor_chain_path).ok()
        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    (base, chain)
}

/// Legacy cursor writing (for backward compatibility)
pub fn write_cursor(paths: &Paths, base_db_abs: Option<&str>, chain_full: Option<&str>) -> Result<()> {
    if let Some(b) = base_db_abs { 
        if !b.is_empty() { 
            fs::write(&paths.cursor_base_path, b)
                .map_err(|e| BookdbError::Io(format!("Failed to write cursor base: {}", e)))?; 
        } 
    }
    if let Some(c) = chain_full { 
        if !c.is_empty() { 
            fs::write(&paths.cursor_chain_path, c)
                .map_err(|e| BookdbError::Io(format!("Failed to write cursor chain: {}", e)))?; 
        } 
    }
    Ok(())
}



// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        // Note: This will need to be adjusted based on your actual Config implementation
        let mut config = Config::default();
        // config.set_data_dir(temp_dir.path().to_path_buf()); // TODO: Adjust based on real Config
        (config, temp_dir)
    }
    
    #[test]
    fn test_cursor_state_persistence() -> Result<()> {
        let (config, _temp) = create_test_config();
        
        // Create and save cursor state
        let mut cursor_state = CursorState::default();
        cursor_state.base_cursor = "test_base".to_string();
        
        cursor_state.save_to_disk(&config)?;
        
        // Load and verify
        let loaded_state = CursorState::load_from_disk(&config)?;
        assert_eq!(loaded_state.base_cursor, "test_base");
        
        Ok(())
    }
    
    #[test]
    fn test_cursor_state_default_when_file_missing() -> Result<()> {
        let (config, _temp) = create_test_config();
        
        // Load from non-existent file should return default
        let cursor_state = CursorState::load_from_disk(&config)?;
        assert_eq!(cursor_state.base_cursor, "home");
        assert!(cursor_state.context_cursor.is_none());
        
        Ok(())
    }
    
    #[test]
    fn test_update_context() {
        let mut cursor_state = CursorState::default();
        
        // Create a test context
        use super::super::context::parse_context_chain;
        let context = parse_context_chain("@test@proj.workspace.var.keystore", "home").unwrap();
        
        cursor_state.update_context(&context);
        
        assert_eq!(cursor_state.base_cursor, "test");
        assert!(cursor_state.context_cursor.is_some());
        assert_eq!(cursor_state.context_cursor.as_ref().unwrap().project, "proj");
    }
    
    #[test]
    fn test_get_current_context_with_fallback() {
        let cursor_state = CursorState::default(); // No context set
        
        let current = cursor_state.get_current_context();
        
        // Should return invincible superchain
        assert_eq!(current.project, "ROOT");
        assert_eq!(current.workspace, "GLOBAL");
        assert_eq!(current.tail, "MAIN");
    }
}
