// src/bookdb/service/ctx/context.rs
// Context chain parsing and display implementations
// Types imported from typesV1, DefaultResolver moved to resolver.rs

use std::fmt;
use crate::error::{Result, BookdbError};

// Import types from typesV1 instead of defining them here
use super::typesV1::{ContextChain, ResolvedContext, Anchor, ChainMode};

// ============================================================================
// DISPLAY IMPLEMENTATIONS
// ============================================================================

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

// ============================================================================
// CORE PARSING FUNCTION
// ============================================================================

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

// ============================================================================
// TESTS
// ============================================================================

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
