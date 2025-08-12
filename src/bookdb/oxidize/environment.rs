// src/bookdb/oxidize/environment.rs - Environment variable bridge for BashFX compatibility
use crate::cli::Cli;

/// Set up environment variables from CLI flags (BashFX compatibility pattern)
/// 
/// This function creates the bridge between CLI flags and environment variables
/// that rdx-stderr and other ODX-compliant libraries expect. It follows exact
/// BashFX patterns for flag interaction and environment variable naming.
pub fn setup_environment_from_flags(cli: &Cli) {
    // === BashFX Standard Environment Variables ===
    
    // Master dev flag: -D enables both debug and trace
    if cli.dev {
        std::env::set_var("DEBUG_MODE", "0");  // BashFX: 0 = enabled
        std::env::set_var("TRACE_MODE", "0");  // -D enables -t
        std::env::set_var("DEV_MODE", "0");    // DEV_MODE marker
    }
    
    // Individual flag handling
    if cli.debug {
        std::env::set_var("DEBUG_MODE", "0");
    }
    
    if cli.trace {
        std::env::set_var("TRACE_MODE", "0");
        // BashFX Rule: -t enables -d as well
        std::env::set_var("DEBUG_MODE", "0");
    }
    
    if cli.quiet {
        std::env::set_var("QUIET_MODE", "0");
    }
    
    if cli.force {
        std::env::set_var("FORCE_MODE", "0");
    }
    
    if cli.yes {
        std::env::set_var("YES_MODE", "0");
    }
    
    // === BookDB-Specific Environment Variables ===
    if cli.dry_run {
        std::env::set_var("BOOKDB_DRY_RUN", "1");
    }
    
    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }
    
    if cli.json {
        std::env::set_var("BOOKDB_JSON_OUTPUT", "1");
    }
    
    // Database configuration
    if let Some(ref db_path) = cli.db_path {
        std::env::set_var("BOOKDB_DATABASE_PATH", db_path);
    }
    
    if let Some(ref base) = cli.base_context {
        std::env::set_var("BOOKDB_BASE_CONTEXT", base);
    }
}

/// Get environment variable with BashFX-style defaults
/// 
/// BashFX Pattern: "0" means enabled, anything else is disabled
/// This maintains compatibility with BashFX shell scripts
pub fn get_env_flag(var_name: &str) -> bool {
    match std::env::var(var_name) {
        Ok(value) => {
            // BashFX pattern: "0" means enabled, anything else is disabled
            value == "0" || value.to_lowercase() == "true"
        }
        Err(_) => false,
    }
}

/// Get environment variable as string with fallback
pub fn get_env_string(var_name: &str, default: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| default.to_string())
}

/// Check if we're in development mode (multiple ways to enable)
pub fn is_dev_mode() -> bool {
    get_env_flag("DEV_MODE") || 
    get_env_flag("BOOKDB_DEV") ||
    (get_env_flag("DEBUG_MODE") && get_env_flag("TRACE_MODE"))
}

/// Check current logging level based on environment (BashFX QUIET(n) compatibility)
pub fn get_current_log_level() -> &'static str {
    if get_env_flag("QUIET_MODE") {
        "error"  // QUIET(1): Only errors and fatals
    } else if get_env_flag("TRACE_MODE") {
        "trace"  // QUIET(3): All messages including trace/think
    } else if get_env_flag("DEBUG_MODE") {
        "info"   // QUIET(2): First-level verbose
    } else {
        "error"  // QUIET(1): Semi-quiet default
    }
}

/// Get current QUIET level as integer
pub fn get_current_quiet_level() -> u8 {
    if get_env_flag("QUIET_MODE") {
        1  // Explicit quiet
    } else if get_env_flag("TRACE_MODE") {
        3  // Trace mode
    } else if get_env_flag("DEBUG_MODE") {
        2  // Debug mode
    } else {
        1  // Semi-quiet default
    }
    // Note: QUIET(0) would require explicit QUIET_MODE=0 AND special handling
}

/// Print environment diagnostic info (for debugging ODX setup)
pub fn print_environment_status() {
    eprintln!("=== ODX Environment Status ===");
    eprintln!("DEBUG_MODE: {}", get_env_flag("DEBUG_MODE"));
    eprintln!("TRACE_MODE: {}", get_env_flag("TRACE_MODE"));
    eprintln!("QUIET_MODE: {}", get_env_flag("QUIET_MODE"));
    eprintln!("FORCE_MODE: {}", get_env_flag("FORCE_MODE"));
    eprintln!("YES_MODE: {}", get_env_flag("YES_MODE"));
    eprintln!("DEV_MODE: {}", get_env_flag("DEV_MODE"));
    eprintln!("Current Log Level: {}", get_current_log_level());
    eprintln!("Current QUIET Level: QUIET({})", get_current_quiet_level());
    eprintln!("Is Dev Mode: {}", is_dev_mode());
    eprintln!("==============================");
}

/// Check if BookDB-specific modes are enabled
pub fn is_dry_run_mode() -> bool {
    get_env_flag("BOOKDB_DRY_RUN")
}

pub fn is_json_output_mode() -> bool {
    get_env_flag("BOOKDB_JSON_OUTPUT")
}

pub fn is_no_color_mode() -> bool {
    get_env_flag("NO_COLOR")
}

/// Get BookDB-specific configuration from environment
pub fn get_bookdb_db_path() -> Option<String> {
    std::env::var("BOOKDB_DATABASE_PATH").ok()
}

pub fn get_bookdb_base_context() -> Option<String> {
    std::env::var("BOOKDB_BASE_CONTEXT").ok()
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
            dev: false,
            dry_run: false,
            no_color: false,
            json: false,
            db_path: None,
            base_context: None,
            command: Commands::Stats { context_chain: None },
        }
    }
    
    #[test]
    fn test_debug_flag_sets_environment() {
        // Clear any existing env vars
        std::env::remove_var("DEBUG_MODE");
        
        let mut cli = mock_cli();
        cli.debug = true;
        
        setup_environment_from_flags(&cli);
        
        assert!(get_env_flag("DEBUG_MODE"));
        assert!(!get_env_flag("TRACE_MODE"));
        assert_eq!(get_current_log_level(), "info");
        assert_eq!(get_current_quiet_level(), 2);
    }
    
    #[test]
    fn test_trace_enables_debug() {
        // Clear any existing env vars
        std::env::remove_var("DEBUG_MODE");
        std::env::remove_var("TRACE_MODE");
        
        let mut cli = mock_cli();
        cli.trace = true;
        
        setup_environment_from_flags(&cli);
        
        assert!(get_env_flag("DEBUG_MODE")); // -t enables -d
        assert!(get_env_flag("TRACE_MODE"));
        assert_eq!(get_current_log_level(), "trace");
        assert_eq!(get_current_quiet_level(), 3);
    }
    
    #[test]
    fn test_dev_flag_enables_both() {
        // Clear any existing env vars
        std::env::remove_var("DEBUG_MODE");
        std::env::remove_var("TRACE_MODE");
        std::env::remove_var("DEV_MODE");
        
        let mut cli = mock_cli();
        cli.dev = true;
        
        setup_environment_from_flags(&cli);
        
        assert!(get_env_flag("DEBUG_MODE"));
        assert!(get_env_flag("TRACE_MODE"));
        assert!(get_env_flag("DEV_MODE"));
        assert!(is_dev_mode());
    }
    
    #[test]
    fn test_bashfx_env_pattern() {
        // Test BashFX pattern: "0" means enabled
        std::env::set_var("TEST_FLAG", "0");
        assert!(get_env_flag("TEST_FLAG"));
        
        std::env::set_var("TEST_FLAG", "1");
        assert!(!get_env_flag("TEST_FLAG"));
        
        std::env::set_var("TEST_FLAG", "true");
        assert!(get_env_flag("TEST_FLAG"));
        
        std::env::remove_var("TEST_FLAG");
        assert!(!get_env_flag("TEST_FLAG"));
    }
    
    #[test]
    fn test_quiet_level_priority() {
        // Clear environment
        std::env::remove_var("DEBUG_MODE");
        std::env::remove_var("TRACE_MODE");
        std::env::remove_var("QUIET_MODE");
        
        // Default: semi-quiet
        assert_eq!(get_current_log_level(), "error");
        assert_eq!(get_current_quiet_level(), 1);
        
        // Debug mode
        std::env::set_var("DEBUG_MODE", "0");
        assert_eq!(get_current_log_level(), "info");
        assert_eq!(get_current_quiet_level(), 2);
        
        // Trace mode (higher priority)
        std::env::set_var("TRACE_MODE", "0");
        assert_eq!(get_current_log_level(), "trace");
        assert_eq!(get_current_quiet_level(), 3);
        
        // Quiet overrides everything
        std::env::set_var("QUIET_MODE", "0");
        assert_eq!(get_current_log_level(), "error");
        assert_eq!(get_current_quiet_level(), 1);
    }
    
    #[test]
    fn test_bookdb_specific_environment() {
        let mut cli = mock_cli();
        cli.dry_run = true;
        cli.json = true;
        cli.db_path = Some("/test/path.db".to_string());
        
        setup_environment_from_flags(&cli);
        
        assert!(is_dry_run_mode());
        assert!(is_json_output_mode());
        assert_eq!(get_bookdb_db_path(), Some("/test/path.db".to_string()));
    }
}
