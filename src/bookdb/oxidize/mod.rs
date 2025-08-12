// src/bookdb/oxidize/mod.rs - ODX compliance module for BookDB
//! # BookDB Oxidize Module
//! 
//! This module provides ODX (Oxidex) compliance for BookDB, implementing
//! BashFX-compatible patterns and architectural standards.
//! 
//! "Oxidize" means to bring code into ODX compliance by normalizing patterns,
//! applying architectural standards, and ensuring predictable behavior.

pub mod config;
pub mod environment;
pub mod flags;

// Re-export main types for convenience
pub use config::OxidexConfig;
pub use environment::{
    setup_environment_from_flags,
    get_env_flag,
    get_env_string,
    is_dev_mode,
    get_current_log_level,
    print_environment_status,
};
pub use flags::BashFxFlags;

/// Initialize ODX framework for BookDB from CLI (convenience function)
pub fn init_from_cli(cli: &crate::cli::Cli) -> OxidexConfig {
    // Set up environment variables first (for rdx-stderr)
    environment::setup_environment_from_flags(cli);
    
    // Create global config
    config::OxidexConfig::from_cli(cli)
}

/// Quick development mode check (for conditional compilation/features)
pub fn quick_dev_check() -> bool {
    environment::is_dev_mode()
}

/// Oxidize a BookDB operation with standard error handling
pub fn oxidize_operation<F, T>(operation: F, config: &OxidexConfig) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error>>,
{
    let mut logger = stderr::Stderr::new();
    
    if config.show_trace() {
        logger.trace("Starting oxidized operation");
    }
    
    let result = operation();
    
    if config.show_trace() {
        match &result {
            Ok(_) => logger.trace("Oxidized operation completed successfully"),
            Err(e) => logger.trace(&format!("Oxidized operation failed: {}", e)),
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    
    fn mock_cli() -> Cli {
        Cli {
            debug: false,
            trace: false,
            quiet: false,
            force: false,
            yes: false,
            dev: true,  // Enable dev mode for testing
            dry_run: false,
            no_color: false,
            json: false,
            db_path: None,
            base_context: None,
            command: Commands::Stats { context_chain: None },
        }
    }
    
    #[test]
    fn test_odx_initialization() {
        let cli = mock_cli();
        let config = init_from_cli(&cli);
        
        // Dev mode should enable debug and trace
        assert!(config.dev);
        assert!(config.debug);
        assert!(config.trace);
        
        // Environment should be set up
        assert!(get_env_flag("DEV_MODE"));
        assert!(get_env_flag("DEBUG_MODE"));
        assert!(get_env_flag("TRACE_MODE"));
        
        // Logging level should reflect trace mode
        assert_eq!(get_current_log_level(), "trace");
    }
    
    #[test]
    fn test_odx_integration() {
        let cli = mock_cli();
        let _config = init_from_cli(&cli);
        
        // Should be able to detect dev mode
        assert!(quick_dev_check());
        assert!(is_dev_mode());
    }
    
    #[test]
    fn test_oxidize_operation() {
        let cli = mock_cli();
        let config = init_from_cli(&cli);
        
        // Test successful operation
        let result = oxidize_operation(|| Ok(42), &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        // Test failed operation
        let error_result = oxidize_operation(|| Err("test error".into()), &config);
        assert!(error_result.is_err());
    }
}
