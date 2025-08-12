// src/context.rs - Complete rewrite for BOOKDB_CONCEPTS.md compliance
//
// CRITICAL FIXES:
// 1. project.docstore.VAR.varstore → project.workspace.var.keystore
// 2. Add FQCC and CDCC resolution modes
// 3. Add ROOT.GLOBAL.VAR.MAIN invincible superchain support
// 4. Context atomicity enforcement
// 5. Proper chain prefix handling (@, %, #)

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::{Result, BookdbError};

/// Represents a fully parsed Context Chain per BOOKDB_CONCEPTS.md
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextChain {
    /// Base name (from base@ prefix) - None means use cursor default
    pub base: Option<String>,
    /// Project name (top-level container)
    pub project: String,
    /// Workspace name (was incorrectly called "docstore" in old version)
    pub workspace: String,
    /// Anchor type: VAR or DOC (case-insensitive)
    pub anchor: Anchor,
    /// Tail: keystore name (for var) or document_key (for doc)
    pub tail: String,
    /// Chain prefix mode
    pub prefix_mode: ChainMode,
    /// Whether this is a Fully Qualified Context Chain (has explicit base@)
    pub is_fqcc: bool,
}

/// Type specifier anchor per CONCEPTS.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    /// Variable store access (.var.)
    Var,
    /// Document store access (.doc.)
    Doc,
}

/// Chain prefix modes per CONCEPTS.md Section 2b
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainMode {
    /// @ - Persistent: updates cursor permanently
    Persistent,
    /// % - Ephemeral: one-time use, cursor unchanged
    Ephemeral,
    /// # - Action: implicit action mode (ROADMAP)
    Action,
}

/// Cursor state for CDCC resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorState {
    /// Current active base
    pub base_cursor: String,
    /// Current context within that base
    pub context_cursor: Option<ContextChain>,
}

/// Resolved context with all fields populated
#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub base: String,
    pub project: String,
    pub workspace: String,
    pub anchor: Anchor,
    pub tail: String,
    pub prefix_mode: ChainMode,
}

impl fmt::Display for ContextChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      if self.is_fqcc {
        // FQCC: base@project.workspace.anchor.tail
        let base_part = self.base.as_ref().unwrap();
        let anchor_str = match self.anchor {
            Anchor::Var => "var",
            Anchor::Doc => "doc",
        };
        write!(f, "{}@{}.{}.{}.{}", 
                base_part, self.project, self.workspace, anchor_str, self.tail)
      } else {
        // CDCC: prefix+project.workspace.anchor.tail
        let prefix = match self.prefix_mode {
            ChainMode::Persistent => '@',
            ChainMode::Ephemeral => '%',
            ChainMode::Action => '#',
        };
        let anchor_str = match self.anchor {
            Anchor::Var => "var",
            Anchor::Doc => "doc",
        };
        write!(f, "{}{}.{}.{}.{}", 
                prefix, self.project, self.workspace, anchor_str, self.tail)
      }
    }
}

impl fmt::Display for ResolvedContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let anchor_str = match self.anchor {
            Anchor::Var => "var",
            Anchor::Doc => "doc",
        };
        write!(f, "{}.{}.{}.{}", self.project, self.workspace, anchor_str, self.tail)
    }
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState {
            base_cursor: "home".to_string(),
            context_cursor: None,
        }
    }
}

/// Parse a context chain string according to BOOKDB_CONCEPTS.md
pub fn parse_context_chain(raw: &str, fallback_base: &str) -> Result<ContextChain> {
    if raw.is_empty() {
        return Err(BookdbError::ContextParse("Empty context chain".to_string()));
    }
    
    // Step 1: Determine prefix mode
    let (prefix_mode, chain_body) = match raw.chars().next().unwrap() {
        '@' => (ChainMode::Persistent, &raw[1..]),
        '%' => (ChainMode::Ephemeral, &raw[1..]),
        '#' => (ChainMode::Action, &raw[1..]),
        _ => return Err(BookdbError::ContextParse(
            "Context chain must start with @, %, or #".to_string())),
    };
    
    // Step 2: Handle base@ prefix for FQCC vs CDCC
    let (base, rest) = if chain_body.contains('@') {
        // FQCC format: base@project.workspace.var.keystore (no prefix on base name)
        let parts: Vec<&str> = chain_body.splitn(2, '@').collect();
        if parts.len() != 2 {
            return Err(BookdbError::ContextParse(
                "Invalid base@ format".to_string()));
        }
        let base_part = parts[0];
        let rest_part = parts[1];
        
        if base_part.is_empty() {
            return Err(BookdbError::ContextParse(
                "Empty base name before @".to_string()));
        }
        
        // Base names cannot have prefixes
        if base_part.starts_with('@') || base_part.starts_with('%') || base_part.starts_with('#') {
            return Err(BookdbError::ContextParse(
                "Base names cannot have prefixes (@, %, #)".to_string()));
        }
        
        (Some(base_part.to_string()), rest_part)
    } else {
        // CDCC format: project.workspace.var.keystore (uses fallback base)
        (None, chain_body)
    };
    
    // Step 3: Split into components: project.workspace.anchor.tail
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() != 4 {
        return Err(BookdbError::ContextParse(format!(
            "Context chain must have exactly 4 parts: project.workspace.anchor.tail, got {} parts", 
            parts.len())));
    }
    
    let project = parts[0].to_string();
    let workspace = parts[1].to_string();
    let anchor_str = parts[2];
    let tail = parts[3].to_string();
    
    // Validate project name
    if project.is_empty() {
        return Err(BookdbError::ContextParse("Empty project name".to_string()));
    }
    
    // Validate workspace name
    if workspace.is_empty() {
        return Err(BookdbError::ContextParse("Empty workspace name".to_string()));
    }
    
    // Parse anchor (case-insensitive per CONCEPTS.md)
    let anchor = match anchor_str.to_lowercase().as_str() {
        "var" | "v" => Anchor::Var,  // v abbreviation for future roadmap
        "doc" | "d" => Anchor::Doc,  // d abbreviation for future roadmap
        _ => return Err(BookdbError::ContextParse(format!(
            "Invalid anchor '{}', must be 'var' or 'doc'", anchor_str))),
    };
    
    // Validate tail
    if tail.is_empty() {
        return Err(BookdbError::ContextParse("Empty tail (keystore/doc_key)".to_string()));
    }
    
    // Check for reserved namespace violations
    if ["var", "doc"].contains(&project.as_str()) ||
       ["var", "doc"].contains(&workspace.as_str()) ||
       ["var", "doc"].contains(&tail.as_str()) {
        return Err(BookdbError::ContextParse(
            "Cannot use 'var' or 'doc' as namespace names".to_string()));
    }
    
    let is_fqcc = base.is_some();
    let final_base = base.or_else(|| Some(fallback_base.to_string()));
    
    Ok(ContextChain {
        base: final_base,
        project,
        workspace,
        anchor,
        tail,
        prefix_mode,
        is_fqcc,
    })
}

/// Default resolver for CDCC and atomicity rules
pub struct DefaultResolver;

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

// Legacy compatibility for existing code
#[derive(Debug, Clone)]
pub enum ResolvedContextIds {
    Variables { vs_id: i64, base_id: i64, proj_id: i64, ds_id: i64 },
    Document { base_id: i64, proj_id: i64, ds_id: i64 },
    PartialVars { base_id: i64, proj_id: i64, ds_id: i64 },
    PartialDocs { base_id: i64, proj_id: i64, ds_id: i64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fqcc_parsing() -> Result<()> {
        // CORRECT: Base has no prefix
        let chain = parse_context_chain("work@website.api_keys.var.credentials", "home")?;
        
        assert_eq!(chain.base, Some("work".to_string()));
        assert_eq!(chain.project, "website");
        assert_eq!(chain.workspace, "api_keys");
        assert_eq!(chain.anchor, Anchor::Var);
        assert_eq!(chain.tail, "credentials");
        assert_eq!(chain.prefix_mode, ChainMode::Persistent);
        assert!(chain.is_fqcc);
        
        Ok(())
    }
    
    #[test]
    fn test_cdcc_parsing() -> Result<()> {
        // CORRECT: CDCC with @ prefix on chain, not base
        let chain = parse_context_chain("@frontend.deployment.var.production", "work")?;
        
        assert_eq!(chain.base, Some("work".to_string()));
        assert_eq!(chain.project, "frontend");
        assert_eq!(chain.workspace, "deployment");
        assert_eq!(chain.anchor, Anchor::Var);
        assert_eq!(chain.tail, "production");
        assert!(!chain.is_fqcc);
        
        Ok(())
    }
    
    #[test]
    fn test_base_prefix_rejection() {
        // INVALID: Base cannot have prefixes
        assert!(parse_context_chain("@work@website.api_keys.var.credentials", "home").is_err());
        assert!(parse_context_chain("%work@website.api_keys.var.credentials", "home").is_err());
        assert!(parse_context_chain("#work@website.api_keys.var.credentials", "home").is_err());
    }
    
    #[test]
    fn test_ephemeral_mode() -> Result<()> {
        let chain = parse_context_chain("%temp@quick.test.var.check", "home")?;
        
        assert_eq!(chain.prefix_mode, ChainMode::Ephemeral);
        assert_eq!(chain.base, Some("temp".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_doc_anchor() -> Result<()> {
        let chain = parse_context_chain("@project.docs.doc.README_md", "home")?;
        
        assert_eq!(chain.anchor, Anchor::Doc);
        assert_eq!(chain.tail, "README_md");
        
        Ok(())
    }
    
    #[test]
    fn test_case_insensitive_anchor() -> Result<()> {
        let chain1 = parse_context_chain("@proj.work.VAR.test", "home")?;
        let chain2 = parse_context_chain("@proj.work.var.test", "home")?;
        let chain3 = parse_context_chain("@proj.work.Var.test", "home")?;
        
        assert_eq!(chain1.anchor, Anchor::Var);
        assert_eq!(chain2.anchor, Anchor::Var);
        assert_eq!(chain3.anchor, Anchor::Var);
        
        Ok(())
    }
    
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
    
    #[test]
    fn test_invalid_chains() {
        // Missing parts
        assert!(parse_context_chain("@proj.workspace.var", "home").is_err());
        
        // Too many parts
        assert!(parse_context_chain("@proj.workspace.var.store.extra", "home").is_err());
        
        // Invalid anchor
        assert!(parse_context_chain("@proj.workspace.invalid.store", "home").is_err());
        
        // Reserved namespace names
        assert!(parse_context_chain("@var.workspace.var.store", "home").is_err());
        assert!(parse_context_chain("@proj.doc.var.store", "home").is_err());
        
        // Missing prefix
        assert!(parse_context_chain("proj.workspace.var.store", "home").is_err());
    }
    
    #[test]
    fn test_display_formatting() -> Result<()> {
        // FQCC: no prefix on base name
        let chain = parse_context_chain("work@website.api_keys.var.credentials", "home")?;
        let display = format!("{}", chain);
        assert_eq!(display, "work@website.api_keys.var.credentials");
        
        // CDCC: prefix on chain, not base
        let ephemeral = parse_context_chain("%temp.test.doc.readme", "home")?;
        let ephemeral_display = format!("{}", ephemeral);
        assert_eq!(ephemeral_display, "%temp.test.doc.readme");
        
        Ok(())
    }
}
