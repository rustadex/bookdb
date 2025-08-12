// src/bookdb/service/ctx/context_manager.rs
// Context manager with stderr integration for rich user experience
// Clean separation - high-level operations only, delegates cursor ops to cursor.rs

use stderr::{Stderr, StderrConfig};
use crate::error::{Result, BookdbError};

// Import from new types structure
use super::typesV1::{ContextChain, ResolvedContext, CursorState, DefaultResolver, Anchor, ChainMode};

// TODO: This import needs to be fixed - find where Config is defined
use crate::config::Config; 
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
    
    /// High-level context resolution - delegates to resolver
    pub fn resolve_context(&self, chain: &ContextChain, cursor_state: &CursorState) -> ResolvedContext {
        self.resolver.resolve_cdcc(chain, cursor_state)
    }
    
    /// Load current cursor state from disk - delegates to CursorState
    pub fn load_cursor_state(&mut self) -> Result<CursorState> {
        self.logger.trace_fn("context_manager", "loading cursor state");
        CursorState::load_from_disk(&self.config)
    }
    
    /// Save cursor state to disk - delegates to CursorState  
    pub fn save_cursor_state(&mut self, cursor_state: &CursorState) -> Result<()> {
        self.logger.trace_fn("context_manager", "saving cursor state");
        cursor_state.save_to_disk(&self.config)
    }
    
    /// Update cursor with atomicity rules and context banner
    pub fn update_cursor(&mut self, new_context: &ContextChain, current_cursors: &mut CursorState) -> Result<()> {
        let old_context = current_cursors.context_cursor.clone();
        
        // Apply atomicity rules if changing from existing context
        let final_context = if let Some(ref old) = old_context {
            self.resolver.apply_atomicity(old, new_context)
        } else {
            new_context.clone()
        };
        
        // Update cursor state (delegates to CursorState)
        current_cursors.update_context(&final_context);
        
        // Show context banner if context changed significantly
        self.show_context_banner_if_changed(&final_context)?;
        
        // Save to disk (delegates to CursorState)
        self.save_cursor_state(current_cursors)?;
        
        Ok(())
    }
    
    /// Show context banner when context changes (private helper)
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
            ChainMode::Persistent => "📌", // Persistent
            ChainMode::Ephemeral => "⚡", // Ephemeral
            ChainMode::Action => "🎯", // Action
        };
        
        let anchor_indicator = match context.anchor {
            Anchor::Var => "🔧", // Variables
            Anchor::Doc => "📄", // Documents
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
                Anchor::Var => "var",
                Anchor::Doc => "doc",
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
                        Anchor::Var => "var",
                        Anchor::Doc => "doc",
                    },
                    match context.anchor {
                        Anchor::Var => "variables",
                        Anchor::Doc => "documents",
                    }
                ),
                &format!("Tail: {}", context.tail),
                &format!("Mode: {} ({})",
                    match context.prefix_mode {
                        ChainMode::Persistent => "@",
                        ChainMode::Ephemeral => "%",
                        ChainMode::Action => "#",
                    },
                    match context.prefix_mode {
                        ChainMode::Persistent => "persistent",
                        ChainMode::Ephemeral => "ephemeral",
                        ChainMode::Action => "action",
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
        // TODO: Update this based on actual Config implementation
        // config.set_data_dir(temp_dir.path().to_path_buf());
        
        let manager = ContextManager::new(config);
        (manager, temp_dir)
    }
    
    #[test]
    fn test_context_banner_display() -> Result<()> {
        let (mut manager, _temp) = create_test_context_manager();
        
        let context = super::super::context::parse_context_chain("work@proj.workspace.var.keystore", "home")?;
        
        // This should not panic
        manager.show_context_banner(&context)?;
        
        Ok(())
    }
    
    #[test]
    fn test_context_manager_delegates_to_cursor_state() -> Result<()> {
        let (mut manager, _temp) = create_test_context_manager();
        
        // Test that cursor operations are delegated properly
        let cursor_state = manager.load_cursor_state()?;
        assert_eq!(cursor_state.base_cursor, "home"); // Default value
        
        Ok(())
    }
}

