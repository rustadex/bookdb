// src/bookdb/service/ctx/v3_validator.rs
// V3 Context Chain Validation and Type Specialization
// Clean, single-purpose functions for segment-based context chains

use stderr::{Stderr, StderrConfig};
use super::typesv3::{ContextChainV3, VarContextChain, DocContextChain, ContextType};
use super::ctx_types::{Anchor, ChainMode};
use crate::error::{Result, BookdbError};

// ============================================================================
// MAIN VALIDATOR FUNCTION
// ============================================================================

/// Validates a context chain string and creates the appropriate V3 type
/// Returns either VarContextChain or DocContextChain based on anchor
pub fn validate_and_create_v3(input: &str, fallback_base: &str) -> Result<V3ContextResult> {
    let mut logger = Stderr::new(&StderrConfig::from_env());
    logger.trace_fn("v3_validator", &format!("validating context chain: '{}'", input));
    
    // Step 1: Parse into generic V3 chain
    let generic_chain = parse_to_generic_v3(input, fallback_base)?;
    
    // Step 2: Specialize based on anchor type
    let specialized = specialize_generic_chain(generic_chain)?;
    
    logger.info(&format!("✅ V3 Context validated: {}", specialized.display_string()));
    Ok(specialized)
}

/// Upgrades a generic ContextChainV3 to the correct specialized type
/// Helper for when you have a ContextChainV3 and want the specific variant
pub fn upgrade_to_specialized(generic: &ContextChainV3) -> Result<V3ContextResult> {
    match generic.chain_type {
        ContextType::Variable => {
            let var_chain = create_var_context_from_generic(generic)?;
            Ok(V3ContextResult::Variable(var_chain))
        }
        ContextType::Document => {
            let doc_chain = create_doc_context_from_generic(generic)?;
            Ok(V3ContextResult::Document(doc_chain))
        }
        ContextType::Mixed => {
            Err(BookdbError::ContextParse("Mixed context types not yet supported".to_string()))
        }
    }
}

// ============================================================================
// RESULT TYPE
// ============================================================================

/// Result of V3 context validation - either Var or Doc specialized type
#[derive(Debug, Clone)]
pub enum V3ContextResult {
    Variable(VarContextChain),
    Document(DocContextChain),
}

impl V3ContextResult {
    /// Get display string for logging
    pub fn display_string(&self) -> String {
        match self {
            V3ContextResult::Variable(var) => format!("{}", var),
            V3ContextResult::Document(doc) => format!("{}", doc),
        }
    }
    
    /// Get the anchor type
    pub fn anchor(&self) -> Anchor {
        match self {
            V3ContextResult::Variable(_) => Anchor::Var,
            V3ContextResult::Document(_) => Anchor::Doc,
        }
    }
    
    /// Get the underlying ContextChainV3 (via Deref)
    pub fn as_generic(&self) -> &ContextChainV3 {
        match self {
            V3ContextResult::Variable(var) => var.deref(),
            V3ContextResult::Document(doc) => doc.deref(),
        }
    }
}

// ============================================================================
// PARSING HELPERS (Single Purpose Functions)
// ============================================================================

/// Parse string into generic ContextChainV3 without specialization
fn parse_to_generic_v3(input: &str, fallback_base: &str) -> Result<ContextChainV3> {
    // Basic validation
    validate_input_string(input)?;
    
    // Parse prefix mode
    let (prefix_mode, body) = parse_prefix_mode(input)?;
    
    // Parse base (FQCC vs CDCC)
    let (base, rest, is_fqcc) = parse_base_component(body, fallback_base)?;
    
    // Parse main components
    let components = parse_main_components(rest)?;
    
    // Create generic V3 chain
    create_generic_v3_chain(base, prefix_mode, components, is_fqcc)
}

/// Determine context type and create specialized variant
fn specialize_generic_chain(generic: ContextChainV3) -> Result<V3ContextResult> {
    match generic.chain_type {
        ContextType::Variable => {
            let var_chain = create_var_context_from_generic(&generic)?;
            Ok(V3ContextResult::Variable(var_chain))
        }
        ContextType::Document => {
            let doc_chain = create_doc_context_from_generic(&generic)?;
            Ok(V3ContextResult::Document(doc_chain))
        }
        ContextType::Mixed => {
            Err(BookdbError::ContextParse("Mixed context types not supported".to_string()))
        }
    }
}

// ============================================================================
// VALIDATION HELPERS (Single Purpose Functions)
// ============================================================================

/// Validate basic input string requirements
fn validate_input_string(input: &str) -> Result<()> {
    if input.trim().is_empty() {
        return Err(BookdbError::ContextParse("Empty context chain".to_string()));
    }
    Ok(())
}

/// Parse and validate prefix mode (@, %, #)
fn parse_prefix_mode(input: &str) -> Result<(ChainMode, &str)> {
    let input = input.trim();
    match input.chars().next() {
        Some('@') => Ok((ChainMode::Persistent, &input[1..])),
        Some('%') => Ok((ChainMode::Ephemeral, &input[1..])),
        Some('#') => Ok((ChainMode::Action, &input[1..])),
        _ => Err(BookdbError::ContextParse(
            "Context chain must start with @, %, or #".to_string()
        )),
    }
}

/// Parse base component for FQCC vs CDCC
fn parse_base_component(body: &str, fallback_base: &str) -> Result<(Option<String>, &str, bool)> {
    if body.contains('@') {
        // FQCC format: base@rest
        let parts: Vec<&str> = body.splitn(2, '@').collect();
        if parts.len() != 2 {
            return Err(BookdbError::ContextParse("Invalid FQCC format".to_string()));
        }
        
        let base_part = parts[0];
        if base_part.is_empty() {
            return Err(BookdbError::ContextParse("Empty base name".to_string()));
        }
        
        validate_component_name("base", base_part)?;
        Ok((Some(base_part.to_string()), parts[1], true))
    } else {
        // CDCC format: use fallback base
        Ok((Some(fallback_base.to_string()), body, false))
    }
}

/// Parse main components: project.workspace.anchor.tail
fn parse_main_components(rest: &str) -> Result<ParsedComponents> {
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() != 4 {
        return Err(BookdbError::ContextParse(
            format!("Expected 4 components (project.workspace.anchor.tail), found {}", parts.len())
        ));
    }
    
    let project = parts[0];
    let workspace = parts[1];
    let anchor_str = parts[2];
    let tail = parts[3];
    
    // Validate each component
    validate_component_name("project", project)?;
    validate_component_name("workspace", workspace)?;
    validate_component_name("tail", tail)?;
    
    // Parse anchor
    let anchor = parse_anchor(anchor_str)?;
    
    Ok(ParsedComponents {
        project: project.to_string(),
        workspace: workspace.to_string(),
        anchor,
        tail: tail.to_string(),
    })
}

/// Parse anchor type (var/doc)
fn parse_anchor(anchor_str: &str) -> Result<Anchor> {
    match anchor_str.to_lowercase().as_str() {
        "var" => Ok(Anchor::Var),
        "doc" => Ok(Anchor::Doc),
        _ => Err(BookdbError::ContextParse(
            format!("Invalid anchor '{}', must be 'var' or 'doc'", anchor_str)
        )),
    }
}

/// Validate component name (alphanumeric, underscore, hyphen only)
fn validate_component_name(component_type: &str, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(BookdbError::ContextParse(
            format!("{} cannot be empty", component_type)
        ));
    }
    
    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            return Err(BookdbError::ContextParse(
                format!("Invalid character '{}' in {}", ch, component_type)
            ));
        }
    }
    
    Ok(())
}

// ============================================================================
// CREATION HELPERS (Single Purpose Functions)
// ============================================================================

/// Create generic ContextChainV3 from parsed components
fn create_generic_v3_chain(
    base: Option<String>,
    prefix_mode: ChainMode,
    components: ParsedComponents,
    is_fqcc: bool,
) -> Result<ContextChainV3> {
    let chain_type = match components.anchor {
        Anchor::Var => ContextType::Variable,
        Anchor::Doc => ContextType::Document,
    };
    
    // Create segments (stub implementation for now)
    let segments = vec![]; // TODO: Implement segment creation
    
    Ok(ContextChainV3 {
        segments,
        chain_type,
        is_fqcc,
    })
}

/// Create VarContextChain from generic ContextChainV3
fn create_var_context_from_generic(generic: &ContextChainV3) -> Result<VarContextChain> {
    // Extract fields from segments (stub implementation)
    // TODO: Implement proper segment extraction
    
    Ok(VarContextChain {
        inner: generic.clone(),
        base: None, // TODO: Extract from segments
        prefix_mode: ChainMode::Persistent, // TODO: Extract from segments
        project: "TODO".to_string(), // TODO: Extract from segments
        workspace: "TODO".to_string(), // TODO: Extract from segments
        keystore: "TODO".to_string(), // TODO: Extract from segments
        is_fqcc: generic.is_fqcc,
    })
}

/// Create DocContextChain from generic ContextChainV3
fn create_doc_context_from_generic(generic: &ContextChainV3) -> Result<DocContextChain> {
    // Extract fields from segments (stub implementation)
    // TODO: Implement proper segment extraction
    
    Ok(DocContextChain {
        inner: generic.clone(),
        base: None, // TODO: Extract from segments
        prefix_mode: ChainMode::Persistent, // TODO: Extract from segments
        project: "TODO".to_string(), // TODO: Extract from segments
        workspace: "TODO".to_string(), // TODO: Extract from segments
        document_key: "TODO".to_string(), // TODO: Extract from segments
        is_fqcc: generic.is_fqcc,
    })
}

// ============================================================================
// HELPER TYPES
// ============================================================================

/// Intermediate structure for parsed components
#[derive(Debug)]
struct ParsedComponents {
    project: String,
    workspace: String,
    anchor: Anchor,
    tail: String,
}

// ============================================================================
// NOTES
// ============================================================================

/*
VALIDATOR DESIGN PRINCIPLES:

1. Single Purpose Functions:
   - Each function does ONE specific task
   - Clear separation of parsing, validation, creation
   - Easy to test and debug

2. Clean Call Chain:
   validate_and_create_v3() -> 
     parse_to_generic_v3() -> 
       validate_input_string()
       parse_prefix_mode()
       parse_base_component()
       parse_main_components()
       create_generic_v3_chain()
     specialize_generic_chain() ->
       create_var_context_from_generic() OR
       create_doc_context_from_generic()

3. Helper Functions Available:
   - upgrade_to_specialized() for existing ContextChainV3
   - Individual validation functions can be used separately

4. Stub Implementation:
   - Segment creation/extraction is stubbed
   - Ready for Phase 1.7 implementation
   - Structure supports future segment system

USAGE EXAMPLES:
```rust
// Validate string input
let result = validate_and_create_v3("@work@proj.workspace.var.keystore", "home")?;

// Upgrade existing generic chain
let specialized = upgrade_to_specialized(&generic_chain)?;

// Use anywhere ContextChainV3 is expected (via Deref)
process_context(&result.as_var_chain()); // automatically coerces
```

// String validation → specialized type
let result = validate_and_create_v3("@work@proj.workspace.var.secrets", "home")?;

// Upgrade existing generic chain
let specialized = upgrade_to_specialized(&generic_chain)?;

// Use anywhere ContextChainV3 is expected (automatic coercion via Deref)
fn process_context(ctx: &ContextChainV3) { ... }
process_context(&var_chain); // ✅ Works automatically!


*/
