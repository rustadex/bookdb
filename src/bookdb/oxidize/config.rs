// src/bookdb/oxidize/config.rs - Global configuration following BashFX patterns
use crate::cli::Cli;

/// Global configuration state following ODX/BashFX patterns
/// 
/// This struct maintains the runtime configuration derived from CLI flags
/// and provides methods for checking message visibility levels according
/// to BashFX QUIET(n) specifications.
#[derive(Debug, Clone)]
pub struct OxidexConfig {
    // === BashFX Standard Modes ===
    pub debug: bool,        // First-level verbose (info, warn, okay)
    pub trace: bool,        // Second-level verbose (trace, think)
    pub quiet: bool,        // Silent except error/fatal
    pub force: bool,        // Bypass safety guards
    pub yes: bool,          // Auto-confirm prompts
    pub dev: bool,          // Master dev flag
    
    // === ODX Extensions ===
    pub dry_run: bool,      // Show but don't execute
    pub no_color: bool,     // Disable colors
    pub json: bool,         // Machine-readable output
    
    // === BookDB-Specific Config ===
    pub db_path: Option<String>,
    pub base_context: Option<String>,
}

impl OxidexConfig {
    /// Create config from CLI flags following BashFX interaction rules
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            // BashFX Rule: -D enables both debug and trace
            // BashFX Rule: -t enables debug as well
            debug: cli.debug || cli.dev || cli.trace,
            trace: cli.trace || cli.dev,
            quiet: cli.quiet,
            force: cli.force,
            yes: cli.yes,
            dev: cli.dev,
            
            // ODX Extensions
            dry_run: cli.dry_run,
            no_color: cli.no_color,
            json: cli.json,
            
            // BookDB-specific
            db_path: cli.db_path.clone(),
            base_context: cli.base_context.clone(),
        }
    }
    
    /// Check if we should show first-level messages (BashFX QUIET(2) level)
    pub fn show_info(&self) -> bool {
        !self.quiet && self.debug
    }
    
    /// Check if we should show second-level messages (BashFX QUIET(3) level)
    pub fn show_trace(&self) -> bool {
        !self.quiet && self.trace
    }
    
    /// Check if we should prompt user (BashFX pattern)
    pub fn should_confirm(&self) -> bool {
        !self.yes && !self.quiet && !self.dry_run
    }
    
    /// Get logging level for stderr (BashFX QUIET(n) compatibility)
    pub fn get_log_level(&self) -> &'static str {
        if self.quiet {
            "error"  // QUIET(1): Only errors and fatals
        } else if self.trace {
            "trace"  // QUIET(3): All messages including trace/think
        } else if self.debug {
            "info"   // QUIET(2): First-level: info, warn, okay, error, fatal
        } else {
            "error"  // QUIET(1): Semi-quiet default (BashFX default)
        }
    }
    
    /// Should we skip safety confirmations? (BashFX -f pattern)
    pub fn bypass_safety(&self) -> bool {
        self.force || self.yes || self.dry_run
    }
    
    /// Get effective database path with fallback
    pub fn get_db_path(&self, default: &str) -> &str {
        self.db_path.as_deref().unwrap_or(default)
    }
    
    /// Get effective base context with fallback
    pub fn get_base_context(&self, default: &str) -> &str {
        self.base_context.as_deref().unwrap_or(default)
    }
    
    /// Format for human display vs machine capture (BashFX stream separation)
    pub fn format_for_humans(&self) -> bool {
        !self.json
    }
    
    /// Check if running in development mode
    pub fn is_dev_mode(&self) -> bool {
        self.dev
    }
    
    /// Get QUIET level as integer for compatibility
    pub fn get_quiet_level(&self) -> u8 {
        match (self.quiet, self.debug, self.trace, self.dev) {
            (true, _, _, _) => 1,           // QUIET(1): Explicit quiet mode
            (false, false, false, false) => 1, // QUIET(1): Semi-quiet default
            (false, true, false, _) => 2,   // QUIET(2): Debug mode
            (false, _, true, _) => 3,       // QUIET(3): Trace mode  
            (false, _, _, true) => 4,       // QUIET(4): Dev mode
        }
        // Note: QUIET(0) is only achievable via environment variable
    }
}

impl Default for OxidexConfig {
    fn default() -> Self {
        Self {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    
    fn mock_cli_with_flags(debug: bool, trace: bool, quiet: bool, dev: bool) -> Cli {
        Cli {
            debug,
            trace,
            quiet,
            force: false,
            yes: false,
            dev,
            dry_run: false,
            no_color: false,
            json: false,
            db_path: None,
            base_context: None,
            command: Commands::Stats { context_chain: None },
        }
    }
    
    #[test]
    fn test_bashfx_debug_flag() {
        let cli = mock_cli_with_flags(true, false, false, false);
        let config = OxidexConfig::from_cli(&cli);
        
        assert!(config.debug);
        assert!(!config.trace);
        assert!(config.show_info());
        assert!(!config.show_trace());
        assert_eq!(config.get_log_level(), "info");
        assert_eq!(config.get_quiet_level(), 2);
    }
    
    #[test]
    fn test_bashfx_trace_enables_debug() {
        let cli = mock_cli_with_flags(false, true, false, false);
        let config = OxidexConfig::from_cli(&cli);
        
        assert!(config.debug); // -t enables -d in BashFX
        assert!(config.trace);
        assert!(config.show_info());
        assert!(config.show_trace());
        assert_eq!(config.get_log_level(), "trace");
        assert_eq!(config.get_quiet_level(), 3);
    }
    
    #[test]
    fn test_bashfx_dev_flag() {
        let cli = mock_cli_with_flags(false, false, false, true);
        let config = OxidexConfig::from_cli(&cli);
        
        assert!(config.debug); // -D enables -d
        assert!(config.trace); // -D enables -t
        assert!(config.dev);
        assert!(config.show_info());
        assert!(config.show_trace());
        assert!(config.is_dev_mode());
        assert_eq!(config.get_quiet_level(), 4);
    }
    
    #[test]
    fn test_bashfx_quiet_overrides() {
        let cli = mock_cli_with_flags(true, true, true, false);
        let config = OxidexConfig::from_cli(&cli);
        
        assert!(config.debug);
        assert!(config.trace);
        assert!(config.quiet);
        assert!(!config.show_info());  // quiet overrides debug
        assert!(!config.show_trace()); // quiet overrides trace
        assert_eq!(config.get_log_level(), "error");
        assert_eq!(config.get_quiet_level(), 1);
    }
    
    #[test]
    fn test_default_semi_quiet() {
        let cli = mock_cli_with_flags(false, false, false, false);
        let config = OxidexConfig::from_cli(&cli);
        
        assert!(!config.debug);
        assert!(!config.trace);
        assert!(!config.quiet);
        assert!(!config.show_info());
        assert!(!config.show_trace());
        assert_eq!(config.get_log_level(), "error"); // Semi-quiet default
        assert_eq!(config.get_quiet_level(), 1);
    }
    
    #[test]
    fn test_bookdb_specific_config() {
        let mut cli = mock_cli_with_flags(false, false, false, false);
        cli.db_path = Some("/custom/path.db".to_string());
        cli.base_context = Some("custom_base".to_string());
        
        let config = OxidexConfig::from_cli(&cli);
        
        assert_eq!(config.get_db_path("default.db"), "/custom/path.db");
        assert_eq!(config.get_base_context("default"), "custom_base");
    }
}
