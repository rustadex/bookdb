// src/bookdb/service/ctx/resolver.rs
// DefaultResolver implementation - Extracted from context.rs
// Handles CDCC resolution and context atomicity rules

use super::typesV1::{ContextChain, ResolvedContext, CursorState, DefaultResolver, Anchor, ChainMode};

impl DefaultResolver {
    pub fn new() -> Self {
        DefaultResolver
    }
    
    /// Resolve a Cursor-Dependent Context Chain using cursor defaults
    pub fn resolve_cdcc(&self, partial: &ContextChain, cursors: &CursorState) -> ResolvedContext {
        // Use cursor defaults for missing base
        let base = partial.base.as_ref()
            .unwrap_or(&cursors.base_cursor)
            .clone();
        
        ResolvedContext {
            base,
            project: partial.project.clone(),
            workspace: partial.workspace.clone(),
            anchor: partial.anchor,
            tail: partial.tail.clone(),
            prefix_mode: partial.prefix_mode,
        }
    }
    
    /// Apply context atomicity rules per CONCEPTS.md
    /// When parent context changes, children should reset to defaults
    pub fn apply_atomicity(&self, old_context: &ContextChain, new_context: &ContextChain) -> ContextChain {
        let mut result = new_context.clone();
        
        // Rule: If project changes, reset workspace and tail to defaults
        if old_context.project != new_context.project {
            result.workspace = "GLOBAL".to_string();
            result.tail = "MAIN".to_string();
        }
        // Rule: If workspace changes but project same, reset tail to default
        else if old_context.workspace != new_context.workspace {
            result.tail = "MAIN".to_string();
        }
        
        result
    }
    
    /// Create the invincible superchain: ROOT.GLOBAL.VAR.MAIN
    pub fn create_invincible_superchain(base: &str) -> ContextChain {
        ContextChain {
            base: Some(base.to_string()),
            project: "ROOT".to_string(),
            workspace: "GLOBAL".to_string(),
            anchor: Anchor::Var,
            tail: "MAIN".to_string(),
            prefix_mode: ChainMode::Persistent,
            is_fqcc: true,
        }
    }
    
    /// Check if a context chain represents the invincible superchain
    pub fn is_invincible_superchain(&self, context: &ContextChain) -> bool {
        context.project == "ROOT" &&
        context.workspace == "GLOBAL" &&
        matches!(context.anchor, Anchor::Var) &&
        context.tail == "MAIN"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use super::super::context::parse_context_chain;
    
    #[test]
    fn test_invincible_superchain() {
        let superchain = DefaultResolver::create_invincible_superchain("home");
        let resolver = DefaultResolver::new();
        
        assert_eq!(superchain.project, "ROOT");
        assert_eq!(superchain.workspace, "GLOBAL");
        assert_eq!(superchain.anchor, Anchor::Var);
        assert_eq!(superchain.tail, "MAIN");
        assert!(resolver.is_invincible_superchain(&superchain));
    }
    
    #[test]
    fn test_context_atomicity() -> Result<()> {
        let resolver = DefaultResolver::new();
        
        let old_context = parse_context_chain("@proj1.workspace1.var.store1", "work")?;
        let new_context = parse_context_chain("@proj2.workspace1.var.store1", "work")?;
        
        // Changing project should reset to MAIN keystore, not keep store1
        let resolved = resolver.apply_atomicity(&old_context, &new_context);
        assert_eq!(resolved.workspace, "GLOBAL"); // Reset to default
        assert_eq!(resolved.tail, "MAIN"); // Reset to default
        
        Ok(())
    }
    
    #[test]
    fn test_workspace_change_atomicity() -> Result<()> {
        let resolver = DefaultResolver::new();
        
        let old_context = parse_context_chain("@proj1.workspace1.var.store1", "work")?;
        let new_context = parse_context_chain("@proj1.workspace2.var.store1", "work")?;
        
        // Changing workspace should reset tail to MAIN
        let resolved = resolver.apply_atomicity(&old_context, &new_context);
        assert_eq!(resolved.project, "proj1"); // Same project
        assert_eq!(resolved.workspace, "workspace2"); // New workspace
        assert_eq!(resolved.tail, "MAIN"); // Reset to default
        
        Ok(())
    }
    
    #[test]
    fn test_cdcc_resolution() -> Result<()> {
        let chain = parse_context_chain("@proj.workspace.var.store", "fallback")?;
        let cursors = CursorState {
            base_cursor: "work".to_string(),
            context_cursor: None,
        };
        
        let resolver = DefaultResolver::new();
        let resolved = resolver.resolve_cdcc(&chain, &cursors);
        
        assert_eq!(resolved.base, "fallback"); // Uses chain base, not cursor
        assert_eq!(resolved.project, "proj");
        assert_eq!(resolved.workspace, "workspace");
        assert_eq!(resolved.tail, "store");
        
        Ok(())
    }
}
