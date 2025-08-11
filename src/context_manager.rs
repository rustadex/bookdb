// src/context_manager.rs - Stderr integration with context management
//
// FEATURES:
// 1. Context banners on cursor changes
// 2. Hierarchical tracing for all operations
// 3. Table formatting for ls commands
// 4. Interactive confirmations for destructive operations

use crate::error::{Result, BookdbError};
use crate::context::{ContextChain, ResolvedContext, CursorState, DefaultResolver};
use crate::rdx::stderr::{Stderr, StderrConfig, BorderStyle};
use crate::config::Config;
use std::path::Path;
use serde::{Serialize, Deserialize};

/// Context manager with stderr integration for rich user experience
pub struct ContextManager {
    config: Config,
    logger: Stderr,
    resolver: DefaultResolver,
    last_displayed_context: Option<String>,
}

impl ContextManager {
    pub fn new(config: Config) -> Self {
        let logger = Stderr::new(&StderrConfig::from_env());
        Self {
            config,
            logger,
            resolver: DefaultResolver::new(),
            last_displayed_context: None,
        }
    }
    
    /// Load current cursor state from disk
    pub fn load_cursor_state(&mut self) -> Result<CursorState> {
        self.logger.trace_fn("context_manager", "loading cursor state");
        
        let cursor_file = self.config.get_cursor_file_path();
        
        if !cursor_file.exists() {
            self.logger.trace_fn("context_manager", "no cursor file found, using defaults");
            return Ok(CursorState::default());
        }
        
        let content = std::fs::read_to_string(&cursor_file)?;
        let cursor_state: CursorState = serde_json::from_str(&content)
            .map_err(|e| BookdbError::ConfigParse(format!("Invalid cursor file: {}", e)))?;
        
        self.logger.trace_fn("context_manager", &format!("loaded cursor: base={}", cursor_state.base_cursor));
        Ok(cursor_state)
    }
    
    /// Save cursor state to disk
    pub fn save_cursor_state(&mut self, cursor_state: &CursorState) -> Result<()> {
        self.logger.trace_fn("context_manager", "saving cursor state");
        
        let cursor_file = self.config.get_cursor_file_path();
        
        // Ensure parent directory exists
        if let Some(parent) = cursor_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(cursor_state)?;
        std::fs::write(&cursor_file, content)?;
        
        self.logger.trace_fn("context_manager", "cursor state saved");
        Ok(())
    }
    
    /// Update cursor and show context banner if context changed
    pub fn update_cursor(&mut self, new_context: &ContextChain, current_cursors: &mut CursorState) -> Result<()> {
        let old_context = current_cursors.context_cursor.clone();
        
        // Apply atomicity rules if changing from existing context
        let final_context = if let Some(ref old) = old_context {
            self.resolver.apply_atomicity(old, new_context)
        } else {
            new_context.clone()
        };
        
        // Update base cursor if explicitly specified
        if let Some(ref base) = final_context.base {
            current_cursors.base_cursor = base.clone();
        }
        
        // Update context cursor
        current_cursors.context_cursor = Some(final_context.clone());
        
        // Show context banner if context changed significantly
        self.show_context_banner_if_changed(&final_context)?;
        
        // Save to disk
        self.save_cursor_state(current_cursors)?;
        
        Ok(())
    }
    
    /// Show context banner when context changes
    fn show_context_banner_if_changed(&mut self, context: &ContextChain) -> Result<()> {
        let context_display = format!("{}", context);
        
        // Only show banner if context actually changed
        if let Some(ref last) = self.last_displayed_context {
            if last == &context_display {
                return Ok(());
            }
        }
        
        self.show_context_banner(context)?;
        self.last_displayed_context = Some(context_display);
        
        Ok(())
    }
    
    /// Display current context banner
    pub fn show_context_banner(&mut self, context: &ContextChain) -> Result<()> {
        let base_part = context.base.as_ref()
            .map(|b| format!("{}@", b))
            .unwrap_or_else(|| "<current>@".to_string());
        
        let mode_indicator = match context.prefix_mode {
            crate::context::ChainMode::Persistent => "📌", // Persistent
            crate::context::ChainMode::Ephemeral => "⚡", // Ephemeral
            crate::context::ChainMode::Action => "🎯", // Action
        };
        
        let anchor_indicator = match context.anchor {
            crate::context::Anchor::Var => "🔧", // Variables
            crate::context::Anchor::Doc => "📄", // Documents
        };
        
        // Create context display
        let context_line = format!(
            "{} {} {}{}.{}.{}.{}",
            mode_indicator,
            anchor_indicator,
            base_part,
            context.project,
            context.workspace,
            match context.anchor {
                crate::context::Anchor::Var => "var",
                crate::context::Anchor::Doc => "doc",
            },
            context.tail
        );
        
        self.logger.banner(&format!("Context: {}", context_line), '-')?;
        
        Ok(())
    }
    
    /// Show cursor status in a nice format
    pub fn show_cursor_status(&mut self, cursor_state: &CursorState) -> Result<()> {
        self.logger.banner("Current Cursor Status", '=')?;
        
        self.logger.info(&format!("Base: {}", cursor_state.base_cursor));
        
        if let Some(ref context) = cursor_state.context_cursor {
            self.logger.info(&format!("Context: {}", context));
            
            // Show context breakdown
            self.logger.info("");
            self.logger.info("Context Breakdown:");
            self.logger.list(&[
                &format!("Project: {}", context.project),
                &format!("Workspace: {}", context.workspace),
                &format!("Anchor: {} ({})", 
                    match context.anchor {
                        crate::context::Anchor::Var => "var",
                        crate::context::Anchor::Doc => "doc",
                    },
                    match context.anchor {
                        crate::context::Anchor::Var => "variables",
                        crate::context::Anchor::Doc => "documents",
                    }
                ),
                &format!("Tail: {}", context.tail),
                &format!("Mode: {} ({})",
                    match context.prefix_mode {
                        crate::context::ChainMode::Persistent => "@",
                        crate::context::ChainMode::Ephemeral => "%",
                        crate::context::ChainMode::Action => "#",
                    },
                    match context.prefix_mode {
                        crate::context::ChainMode::Persistent => "persistent",
                        crate::context::ChainMode::Ephemeral => "ephemeral",
                        crate::context::ChainMode::Action => "action",
                    }
                ),
            ], "→")?;
        } else {
            self.logger.info("Context: <not set>");
            self.logger.info("");
            self.logger.info("Use 'bookdb use @project.workspace.var.keystore' to set a context.");
        }
        
        Ok(())
    }
}

/// Confirmation helper for destructive operations
pub struct DestructiveOpConfirm {
    logger: Stderr,
}

impl DestructiveOpConfirm {
    pub fn new() -> Self {
        Self {
            logger: Stderr::new(&StderrConfig::from_env()),
        }
    }
    
    /// Confirm deletion operations
    pub fn confirm_delete(&mut self, item_type: &str, item_name: &str) -> Result<bool> {
        self.logger.warn(&format!("About to delete {} '{}'", item_type, item_name));
        self.logger.error("This operation cannot be undone!");
        
        let result = self.logger.confirm_builder(&format!("Delete {} '{}'?", item_type, item_name))
            .boxed(true)
            .style(BorderStyle::Heavy)
            .ask()?
            .unwrap_or(false);
        
        if result {
            self.logger.trace_fn("destructive_op", &format!("user confirmed deletion of {}", item_name));
        } else {
            self.logger.trace_fn("destructive_op", "user cancelled deletion");
        }
        
        Ok(result)
    }
    
    /// Confirm import/export operations that might overwrite data
    pub fn confirm_overwrite(&mut self, operation: &str, target: &str) -> Result<bool> {
        self.logger.warn(&format!("About to {} - this may overwrite existing data", operation));
        self.logger.info(&format!("Target: {}", target));
        
        let result = self.logger.confirm(&format!("Continue with {}?", operation))?
            .unwrap_or(false);
        
        if result {
            self.logger.trace_fn("destructive_op", &format!("user confirmed {}", operation));
        } else {
            self.logger.trace_fn("destructive_op", &format!("user cancelled {}", operation));
        }
        
        Ok(result)
    }
    
    /// Confirm reset operations
    pub fn confirm_reset(&mut self, scope: &str) -> Result<bool> {
        self.logger.error(&format!("About to reset {} - this will delete all data in scope!", scope));
        self.logger.error("This operation cannot be undone!");
        
        // Require typing "yes" for reset operations
        self.logger.info("");
        self.logger.info("Type 'yes' to confirm reset:");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let confirmed = input.trim().to_lowercase() == "yes";
        
        if confirmed {
            self.logger.trace_fn("destructive_op", &format!("user confirmed reset of {}", scope));
            self.logger.warn("Reset confirmed. Proceeding...");
        } else {
            self.logger.trace_fn("destructive_op", "user cancelled reset");
            self.logger.info("Reset cancelled.");
        }
        
        Ok(confirmed)
    }
}

/// Table formatter for ls commands with stderr integration
pub struct LsTableFormatter {
    logger: Stderr,
}

impl LsTableFormatter {
    pub fn new() -> Self {
        Self {
            logger: Stderr::new(&StderrConfig::from_env()),
        }
    }
    
    /// Format and display a table of items
    pub fn display_table(&mut self, headers: &[&str], rows: &[Vec<String>], title: Option<&str>) -> Result<()> {
        if let Some(title) = title {
            self.logger.banner(title, '=')?;
        }
        
        if rows.is_empty() {
            self.logger.info("No items found.");
            return Ok(());
        }
        
        // Convert to table format
        let mut table_data = vec![headers.to_vec()];
        for row in rows {
            table_data.push(row.iter().map(|s| s.as_str()).collect());
        }
        
        // Convert to &[&[&str]] format required by simple_table
        let table_refs: Vec<Vec<&str>> = table_data.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect())
            .collect();
        let table_slice: Vec<&[&str]> = table_refs.iter()
            .map(|row| row.as_slice())
            .collect();
        
        self.logger.simple_table(&table_slice)?;
        
        self.logger.info(&format!("Total: {} items", rows.len()));
        
        Ok(())
    }
    
    /// Display variables in a nice table format
    pub fn display_variables(&mut self, variables: &[(String, String)], context: &str) -> Result<()> {
        let title = format!("Variables in {}", context);
        
        let rows: Vec<Vec<String>> = variables.iter()
            .map(|(key, value)| vec![key.clone(), value.clone()])
            .collect();
        
        self.display_table(&["Key", "Value"], &rows, Some(&title))
    }
    
    /// Display projects, workspaces, etc. in table format
    pub fn display_namespaces(&mut self, items: &[String], namespace_type: &str, context: &str) -> Result<()> {
        let title = format!("{} in {}", namespace_type, context);
        
        let rows: Vec<Vec<String>> = items.iter()
            .enumerate()
            .map(|(i, name)| vec![(i + 1).to_string(), name.clone()])
            .collect();
        
        self.display_table(&["#", "Name"], &rows, Some(&title))
    }
}

/// Progress tracking for long operations
pub struct OperationProgress {
    logger: Stderr,
    operation_name: String,
    total_items: Option<usize>,
    current_item: usize,
}

impl OperationProgress {
    pub fn new(operation_name: &str) -> Self {
        let logger = Stderr::new(&StderrConfig::from_env());
        Self {
            logger,
            operation_name: operation_name.to_string(),
            total_items: None,
            current_item: 0,
        }
    }
    
    pub fn set_total(&mut self, total: usize) {
        self.total_items = Some(total);
        self.logger.trace_fn("progress", &format!("{}: starting {} items", self.operation_name, total));
    }
    
    pub fn increment(&mut self, item_name: &str) -> Result<()> {
        self.current_item += 1;
        
        if let Some(total) = self.total_items {
            let percent = (self.current_item as f64 / total as f64 * 100.0) as usize;
            self.logger.info(&format!("[{}/{}] ({:3}%) Processing: {}", 
                self.current_item, total, percent, item_name));
        } else {
            self.logger.info(&format!("[{}] Processing: {}", self.current_item, item_name));
        }
        
        Ok(())
    }
    
    pub fn complete(&mut self) -> Result<()> {
        self.logger.okay(&format!("{} completed successfully! Processed {} items.", 
            self.operation_name, self.current_item));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_context_manager() -> (ContextManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.xdg = crate::config::XdgDirs::new(temp_dir.path()).unwrap();
        
        let manager = ContextManager::new(config);
        (manager, temp_dir)
    }
    
    #[test]
    fn test_cursor_state_persistence() -> Result<()> {
        let (mut manager, _temp) = create_test_context_manager();
        
        // Create and save cursor state
        let mut cursor_state = CursorState::default();
        cursor_state.base_cursor = "test_base".to_string();
        
        manager.save_cursor_state(&cursor_state)?;
        
        // Load and verify
        let loaded_state = manager.load_cursor_state()?;
        assert_eq!(loaded_state.base_cursor, "test_base");
        
        Ok(())
    }
    
    #[test]
    fn test_context_banner_display() -> Result<()> {
        let (mut manager, _temp) = create_test_context_manager();
        
        let context = crate::context::parse_context_chain("@work@proj.workspace.var.keystore", "home")?;
        
        // This should not panic
        manager.show_context_banner(&context)?;
        
        Ok(())
    }
}
