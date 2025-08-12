// src/bookdb/oxidize/flags.rs - BashFX flag behavior and co-flag patterns
use std::collections::HashMap;
use crate::cli::Cli;

/// BashFX-compatible flag state representation
/// 
/// This struct represents the parsed state of BashFX standard flags,
/// providing a clean interface for checking flag interactions and
/// implementing co-flag patterns.
#[derive(Debug, Clone, PartialEq)]
pub struct BashFxFlags {
    pub debug: bool,     // -d: First-level verbose
    pub trace: bool,     // -t: Second-level verbose
    pub quiet: bool,     // -q: Silent mode
    pub force: bool,     // -f: Bypass safety
    pub yes: bool,       // -y: Auto-confirm
    pub dev: bool,       // -D: Master dev flag
}

impl BashFxFlags {
    /// Create BashFX flags from CLI, applying interaction rules
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
        }
    }
    
    /// Get the effective QUIET level following BashFX specification
    pub fn get_quiet_level(&self) -> QuietLevel {
        if self.quiet {
            QuietLevel::Quiet1  // Explicit quiet mode
        } else if self.trace {
            QuietLevel::Quiet3  // Trace mode
        } else if self.debug {
            QuietLevel::Quiet2  // Debug mode
        } else {
            QuietLevel::Quiet1  // Semi-quiet default
        }
        // Note: QUIET(0) is only achievable via environment variable
    }
    
    /// Check if safety bypassing is enabled (BashFX pattern)
    pub fn bypass_safety(&self) -> bool {
        self.force || self.yes
    }
    
    /// Check if user confirmation should be skipped
    pub fn skip_confirmation(&self) -> bool {
        self.yes || self.quiet
    }
    
    /// Get flag combination as string (for debugging)
    pub fn to_flag_string(&self) -> String {
        let mut flags = Vec::new();
        
        if self.debug && !self.trace && !self.dev { flags.push("-d"); }
        if self.trace && !self.dev { flags.push("-t"); }
        if self.quiet { flags.push("-q"); }
        if self.force { flags.push("-f"); }
        if self.yes { flags.push("-y"); }
        if self.dev { flags.push("-D"); }
        
        if flags.is_empty() {
            "(no flags)".to_string()
        } else {
            flags.join(" ")
        }
    }
}

/// BashFX QUIET(n) level specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuietLevel {
    Quiet0 = 0,  // Absolute silence (environment only)
    Quiet1 = 1,  // Semi-quiet default (error, fatal only)
    Quiet2 = 2,  // Debug mode (+ info, warn, okay)
    Quiet3 = 3,  // Trace mode (+ trace, think)
    Quiet4 = 4,  // Dev mode (+ dev messages)
    Quiet5 = 5,  // Everything (theoretical maximum)
}

impl QuietLevel {
    /// Get messages visible at this QUIET level
    pub fn visible_messages(&self) -> &'static [&'static str] {
        match self {
            QuietLevel::Quiet0 => &[],  // Absolute silence
            QuietLevel::Quiet1 => &["error", "fatal"],
            QuietLevel::Quiet2 => &["error", "fatal", "info", "warn", "okay"],
            QuietLevel::Quiet3 => &["error", "fatal", "info", "warn", "okay", "trace", "think"],
            QuietLevel::Quiet4 => &["error", "fatal", "info", "warn", "okay", "trace", "think", "dev"],
            QuietLevel::Quiet5 => &["error", "fatal", "info", "warn", "okay", "trace", "think", "dev", "all"],
        }
    }
    
    /// Check if a message type is visible at this level
    pub fn is_visible(&self, message_type: &str) -> bool {
        self.visible_messages().contains(&message_type)
    }
}

/// Co-flag configuration for custom flag combinations
/// 
/// This allows applications to define custom flag combinations beyond
/// the standard BashFX -D pattern.
#[derive(Debug, Clone)]
pub struct CoFlagConfig {
    pub dev_flag: char,           // Default: 'D'
    pub dev_enables: Vec<String>, // Default: ["debug", "trace"]
    pub custom_combinations: HashMap<char, Vec<String>>,
}

impl CoFlagConfig {
    /// Create BashFX standard co-flag configuration
    pub fn bashfx_default() -> Self {
        let mut custom = HashMap::new();
        custom.insert('D', vec!["debug".into(), "trace".into()]);
        
        Self {
            dev_flag: 'D',
            dev_enables: vec!["debug".into(), "trace".into()],
            custom_combinations: custom,
        }
    }
    
    /// Create custom co-flag configuration
    pub fn custom() -> Self {
        Self {
            dev_flag: 'D',
            dev_enables: vec!["debug".into(), "trace".into()],
            custom_combinations: HashMap::new(),
        }
    }
    
    /// Add a custom co-flag combination
    pub fn add_combination(&mut self, flag: char, enables: Vec<String>) {
        self.custom_combinations.insert(flag, enables);
    }
    
    /// Apply co-flag to a mutable CLI structure (for advanced usage)
    pub fn apply_co_flag(&self, flag: char, cli: &mut Cli) {
        if let Some(enables) = self.custom_combinations.get(&flag) {
            for enable in enables {
                match enable.as_str() {
                    "debug" => cli.debug = true,
                    "trace" => cli.trace = true,
                    "quiet" => cli.quiet = true,
                    "force" => cli.force = true,
                    "yes" => cli.yes = true,
                    "dev" => cli.dev = true,
                    "dry_run" => cli.dry_run = true,
                    "json" => cli.json = true,
                    "no_color" => cli.no_color = true,
                    _ => {} // Unknown flags ignored
                }
            }
        }
    }
}

/// Validate BashFX flag combinations for correctness
pub fn validate_flag_combination(flags: &BashFxFlags) -> Result<(), String> {
    // BashFX Rule: -q overrides other verbose flags
    if flags.quiet && (flags.debug || flags.trace) {
        // This is allowed - quiet overrides verbose flags
    }
    
    // BashFX Rule: -D should enable both debug and trace
    if flags.dev && (!flags.debug || !flags.trace) {
        return Err("Dev flag (-D) should enable both debug and trace".into());
    }
    
    // BashFX Rule: -t should enable debug
    if flags.trace && !flags.debug {
        return Err("Trace flag (-t) should enable debug".into());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    
    fn mock_cli_with_flags(debug: bool, trace: bool, quiet: bool, dev: bool) -> Cli {
        Cli {
            debug, trace, quiet, dev,
            force: false, yes: false, dry_run: false, no_color: false, json: false,
            db_path: None, base_context: None,
            command: Commands::Stats { context_chain: None },
        }
    }
    
    #[test]
    fn test_bashfx_flag_creation() {
        let cli = mock_cli_with_flags(true, false, false, false);
        let flags = BashFxFlags::from_cli(&cli);
        
        assert!(flags.debug);
        assert!(!flags.trace);
        assert!(!flags.quiet);
        assert_eq!(flags.get_quiet_level(), QuietLevel::Quiet2);
    }
    
    #[test]
    fn test_trace_enables_debug() {
        let cli = mock_cli_with_flags(false, true, false, false);
        let flags = BashFxFlags::from_cli(&cli);
        
        assert!(flags.debug); // -t enables -d
        assert!(flags.trace);
        assert_eq!(flags.get_quiet_level(), QuietLevel::Quiet3);
    }
    
    #[test]
    fn test_dev_flag_enables_both() {
        let cli = mock_cli_with_flags(false, false, false, true);
        let flags = BashFxFlags::from_cli(&cli);
        
        assert!(flags.debug); // -D enables -d
        assert!(flags.trace); // -D enables -t
        assert!(flags.dev);
        assert_eq!(flags.get_quiet_level(), QuietLevel::Quiet3);
    }
    
    #[test]
    fn test_quiet_overrides() {
        let cli = mock_cli_with_flags(true, true, true, false);
        let flags = BashFxFlags::from_cli(&cli);
        
        assert!(flags.debug);
        assert!(flags.trace);
        assert!(flags.quiet);
        assert_eq!(flags.get_quiet_level(), QuietLevel::Quiet1); // Quiet overrides
    }
    
    #[test]
    fn test_quiet_level_visibility() {
        assert!(QuietLevel::Quiet1.is_visible("error"));
        assert!(QuietLevel::Quiet1.is_visible("fatal"));
        assert!(!QuietLevel::Quiet1.is_visible("info"));
        
        assert!(QuietLevel::Quiet2.is_visible("info"));
        assert!(QuietLevel::Quiet2.is_visible("warn"));
        assert!(!QuietLevel::Quiet2.is_visible("trace"));
        
        assert!(QuietLevel::Quiet3.is_visible("trace"));
        assert!(QuietLevel::Quiet3.is_visible("think"));
    }
    
    #[test]
    fn test_flag_string_representation() {
        let cli = mock_cli_with_flags(true, false, false, false);
        let flags = BashFxFlags::from_cli(&cli);
        assert_eq!(flags.to_flag_string(), "-d");
        
        let cli = mock_cli_with_flags(false, false, false, true);
        let flags = BashFxFlags::from_cli(&cli);
        assert_eq!(flags.to_flag_string(), "-D");
        
        let cli = mock_cli_with_flags(false, false, false, false);
        let flags = BashFxFlags::from_cli(&cli);
        assert_eq!(flags.to_flag_string(), "(no flags)");
    }
    
    #[test]
    fn test_flag_validation() {
        let cli = mock_cli_with_flags(true, true, false, true);
        let flags = BashFxFlags::from_cli(&cli);
        assert!(validate_flag_combination(&flags).is_ok());
        
        // Test manual invalid combination (shouldn't happen with from_cli)
        let invalid_flags = BashFxFlags {
            debug: false, trace: true, quiet: false,
            force: false, yes: false, dev: false,
        };
        assert!(validate_flag_combination(&invalid_flags).is_err());
    }
    
    #[test]
    fn test_co_flag_config() {
        let config = CoFlagConfig::bashfx_default();
        assert_eq!(config.dev_flag, 'D');
        assert_eq!(config.dev_enables, vec!["debug", "trace"]);
        
        let mut custom_config = CoFlagConfig::custom();
        custom_config.add_combination('V', vec!["debug".into(), "json".into()]);
        
        let mut cli = mock_cli_with_flags(false, false, false, false);
        custom_config.apply_co_flag('V', &mut cli);
        
        assert!(cli.debug);
        assert!(cli.json);
        assert!(!cli.trace);
    }
}
