// src/context_manager.rs - Stderr integration with context management
//
// FEATURES:
// 1. Context banners on cursor changes
// 2. Hierarchical tracing for all operations
// 3. Table formatting for ls commands
// 4. Interactive confirmations for destructive operations

use stderr::{Stderr, StderrConfig};

use crate::error::{Result, BookdbError};
use crate::ctx::{ContextChain, ResolvedContext, CursorState, DefaultResolver};


// todo: find this
use crate::config::Config; //? where is this
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
        
        let context = crate::context::parse_context_chain("work@proj.workspace.var.keystore", "home")?;
        
        // This should not panic
        manager.show_context_banner(&context)?;
        
        Ok(())
    }
}
