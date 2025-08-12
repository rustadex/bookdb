// src/installation.rs - Installation guard to block usage until setup
//
// CRITICAL: Blocks all BookDB usage until 'bookdb install' is run
// Prevents data corruption and ensures proper initialization

use crate::error::{Result, BookdbError};
use crate::context::{ContextChain, DefaultResolver, Anchor, ChainMode};
use crate::db::Database;
use crate::config::Config;
use std::path::Path;
use crate::rdx::stderr::{Stderr, StderrConfig};

/// Installation guard that blocks usage until proper setup
pub struct InstallationGuard {
    config: Config,
    logger: Stderr,
}

impl InstallationGuard {
    pub fn new(config: Config) -> Self {
        let logger = Stderr::new(&StderrConfig::from_env());
        Self { config, logger }
    }
    
    /// CRITICAL: Call this before any BookDB operations
    /// Blocks execution if installation is incomplete
    pub fn require_installation(&mut self) -> Result<()> {
        self.logger.trace_fn("installation_guard", "checking installation status");
        
        // Check if home base exists and is properly initialized
        let home_db_path = self.config.get_base_path("home");
        
        if !home_db_path.exists() {
            self.show_installation_required()?;
            return Err(BookdbError::NotInstalled(
                "BookDB is not installed. Run 'bookdb install' first.".to_string()));
        }
        
        // Check if database has proper meta table with installed flag
        match self.check_installation_meta(&home_db_path) {
            Ok(true) => {
                self.logger.trace_fn("installation_guard", "installation verified");
                Ok(())
            }
            Ok(false) => {
                self.show_installation_incomplete()?;
                Err(BookdbError::NotInstalled(
                    "BookDB installation is incomplete. Run 'bookdb install' to finish setup.".to_string()))
            }
            Err(e) => {
                self.logger.warn(&format!("Failed to check installation status: {}", e));
                self.show_installation_required()?;
                Err(BookdbError::NotInstalled(
                    "BookDB installation is corrupted. Run 'bookdb install' to repair.".to_string()))
            }
        }
    }
    
    /// Check if database has proper installation metadata
    fn check_installation_meta(&mut self, db_path: &Path) -> Result<bool> {
        use rusqlite::Connection;
        
        let conn = Connection::open(db_path)?;
        
        // Check if meta table exists
        let table_exists: bool = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='meta'"
        )?.exists([])?;
        
        if !table_exists {
            return Ok(false);
        }
        
        // Check installation flag
        let installed: Option<i64> = conn.query_row(
            "SELECT value FROM meta WHERE key = 'installed'",
            [],
            |row| Ok(row.get::<_, String>(0)?.parse::<i64>().unwrap_or(0))
        ).optional()?;
        
        Ok(installed == Some(1))
    }
    
    /// Show user-friendly installation guidance
    fn show_installation_required(&mut self) -> Result<()> {
        self.logger.banner("BookDB Not Installed", '!')?;
        self.logger.error("BookDB has not been installed on this system.");
        self.logger.info("");
        self.logger.info("To get started, run:");
        self.logger.info("  bookdb install");
        self.logger.info("");
        self.logger.info("This will:");
        self.logger.list(&[
            "Create the default 'home' base",
            "Initialize the invincible superchain (ROOT.GLOBAL.VAR.MAIN)",
            "Set up your configuration directory",
            "Create necessary database tables",
        ], "→")?;
        
        Ok(())
    }
    
    /// Show guidance for incomplete installation
    fn show_installation_incomplete(&mut self) -> Result<()> {
        self.logger.banner("Incomplete Installation", '!')?;
        self.logger.warn("BookDB installation was started but not completed properly.");
        self.logger.info("");
        self.logger.info("To fix this, run:");
        self.logger.info("  bookdb install");
        self.logger.info("");
        self.logger.info("This will complete the installation process safely.");
        
        Ok(())
    }
}

/// Installation manager for the 'install' command
pub struct InstallationManager {
    config: Config,
    logger: Stderr,
}

impl InstallationManager {
    pub fn new(config: Config) -> Self {
        let logger = Stderr::new(&StderrConfig::from_env());
        Self { config, logger }
    }
    
    /// Execute the installation process
    pub fn install(&mut self) -> Result<()> {
        self.logger.banner("BookDB Installation", '=')?;
        
        // Check if already installed
        if self.is_already_installed()? {
            self.logger.warn("BookDB is already installed.");
            if !self.logger.confirm("Reinstall and reset to defaults?")?.unwrap_or(false) {
                self.logger.info("Installation cancelled.");
                return Ok(());
            }
            self.logger.info("Proceeding with reinstallation...");
        }
        
        // Show installation plan
        self.logger.info("Installation Plan:");
        self.logger.list(&[
            "Create XDG+ directory structure",
            "Initialize default 'home' base",
            "Create invincible superchain (ROOT.GLOBAL.VAR.MAIN)",
            "Set up database schema",
            "Mark installation as complete",
        ], "→")?;
        
        if !self.logger.confirm("Proceed with installation?")?.unwrap_or(false) {
            self.logger.info("Installation cancelled.");
            return Ok(());
        }
        
        // Execute installation steps
        self.perform_installation()?;
        
        self.logger.okay("✓ BookDB installation completed successfully!");
        self.logger.info("");
        self.logger.info("You can now use BookDB. Try:");
        self.logger.info("  bookdb cursor              # Show current context");
        self.logger.info("  bookdb setv hello=world    # Set a variable");
        self.logger.info("  bookdb getv hello          # Get the variable");
        
        Ok(())
    }
    
    /// Check if BookDB is already properly installed
    fn is_already_installed(&mut self) -> Result<bool> {
        let home_db_path = self.config.get_base_path("home");
        
        if !home_db_path.exists() {
            return Ok(false);
        }
        
        let guard = InstallationGuard::new(self.config.clone());
        match guard.check_installation_meta(&home_db_path) {
            Ok(installed) => Ok(installed),
            Err(_) => Ok(false), // Treat errors as not installed
        }
    }
    
    /// Perform the actual installation
    fn perform_installation(&mut self) -> Result<()> {
        self.logger.trace_fn("installation", "starting installation process");
        
        // Step 1: Create directory structure
        self.create_directory_structure()?;
        
        // Step 2: Initialize home base database
        let home_db_path = self.config.get_base_path("home");
        let mut database = Database::create_or_open(&home_db_path)?;
        
        // Step 3: Create invincible superchain
        self.create_invincible_superchain(&mut database)?;
        
        // Step 4: Mark installation complete
        self.mark_installation_complete(&mut database)?;
        
        self.logger.trace_fn("installation", "installation completed successfully");
        Ok(())
    }
    
    /// Create necessary directory structure
    fn create_directory_structure(&mut self) -> Result<()> {
        self.logger.trace_fn("installation", "creating directory structure");
        
        let data_dir = self.config.xdg.data_dir();
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)?;
            self.logger.trace_fn("installation", "created data directory");
        }
        
        let config_dir = self.config.xdg.config_dir();
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir)?;
            self.logger.trace_fn("installation", "created config directory");
        }
        
        Ok(())
    }
    
    /// Create the invincible superchain: ROOT.GLOBAL.VAR.MAIN
    fn create_invincible_superchain(&mut self, database: &mut Database) -> Result<()> {
        self.logger.trace_fn("installation", "creating invincible superchain");
        
        // Create the ROOT.GLOBAL.VAR.MAIN context
        let superchain = DefaultResolver::create_invincible_superchain("home");
        
        // Ensure all necessary namespaces exist
        database.ensure_project_exists("ROOT")?;
        database.ensure_workspace_exists("ROOT", "GLOBAL")?;
        database.ensure_varstore_exists("ROOT", "GLOBAL", "MAIN")?;
        
        // Set a special marker in the invincible superchain
        database.set_variable_in_context("_INVINCIBLE", "1", &superchain)?;
        database.set_variable_in_context("_INSTALLED", "1", &superchain)?;
        database.set_variable_in_context("_VERSION", env!("CARGO_PKG_VERSION"), &superchain)?;
        
        self.logger.trace_fn("installation", "invincible superchain created");
        Ok(())
    }
    
    /// Mark installation as complete in meta table
    fn mark_installation_complete(&mut self, database: &mut Database) -> Result<()> {
        self.logger.trace_fn("installation", "marking installation complete");
        
        // Create meta table if not exists
        database.execute_sql(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )?;
        
        // Set installation flag
        database.execute_sql(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('installed', '1')"
        )?;
        
        // Set installation timestamp
        database.execute_sql(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('installed_at', datetime('now'))"
        )?;
        
        // Set version
        database.execute_sql(&format!(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('version', '{}')",
            env!("CARGO_PKG_VERSION")
        ))?;
        
        self.logger.trace_fn("installation", "installation marked complete");
        Ok(())
    }
}

/// Integration for main.rs to check installation before any operations
pub fn require_installation_or_install(config: &Config, is_install_command: bool) -> Result<()> {
    if is_install_command {
        // Allow install command to proceed
        return Ok(());
    }
    
    let mut guard = InstallationGuard::new(config.clone());
    guard.require_installation()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.xdg = XdgDirs::new(temp_dir.path()).unwrap();
        (config, temp_dir)
    }
    
    #[test]
    fn test_installation_guard_blocks_when_not_installed() {
        let (config, _temp) = create_test_config();
        let mut guard = InstallationGuard::new(config);
        
        // Should fail when not installed
        assert!(guard.require_installation().is_err());
    }
    
    #[test]
    fn test_installation_process() -> Result<()> {
        let (config, _temp) = create_test_config();
        let mut manager = InstallationManager::new(config.clone());
        
        // Perform installation
        manager.perform_installation()?;
        
        // Verify installation
        let mut guard = InstallationGuard::new(config);
        assert!(guard.require_installation().is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_invincible_superchain_creation() -> Result<()> {
        let (config, _temp) = create_test_config();
        let home_db_path = config.get_base_path("home");
        
        // Create database and invincible superchain
        let mut database = Database::create_or_open(&home_db_path)?;
        let mut manager = InstallationManager::new(config);
        manager.create_invincible_superchain(&mut database)?;
        
        // Verify superchain exists and has marker
        let superchain = DefaultResolver::create_invincible_superchain("home");
        let value = database.get_variable_in_context("_INVINCIBLE", &superchain)?;
        assert_eq!(value, Some("1".to_string()));
        
        Ok(())
    }
}
