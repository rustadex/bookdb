// context_validator.rs - Standalone context chain validation and expansion
// 
// This function validates and fully expands context chain strings according to
// BOOKDB_CONCEPTS.md specification, providing detailed error messages via stderr.

use stderr::{Stderr, StderrConfig};

#[derive(Debug, Clone, PartialEq)]
pub enum ChainMode {
    Persistent, // @
    Ephemeral,  // %
    Action,     // #
}

#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    Var, // Variable context
    Doc, // Document context
}

#[derive(Debug, Clone)]
pub struct ExpandedContext {
    pub base: String,
    pub project: String,
    pub workspace: String,
    pub anchor: Anchor,
    pub tail: String,           // keystore name for var, doc name for doc
    pub prefix_mode: ChainMode,
    pub is_fqcc: bool,          // Fully Qualified Context Chain
    pub original_input: String,
    pub canonical_form: String, // Fully expanded canonical representation
}

impl std::fmt::Display for ExpandedContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.canonical_form)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextValidationError {
    #[error("Empty context chain provided")]
    EmptyChain,
    
    #[error("Invalid prefix: context chain must start with @ (persistent), % (ephemeral), or # (action)")]
    InvalidPrefix,
    
    #[error("Invalid base@ format: expected 'base@project.workspace.anchor.tail'")]
    InvalidFqccFormat,
    
    #[error("Empty base name: base cannot be empty in FQCC")]
    EmptyBaseName,
    
    #[error("Base name cannot have prefix: '{0}' contains invalid characters")]
    BaseWithPrefix(String),
    
    #[error("Invalid component count: expected 4 parts (project.workspace.anchor.tail), found {0}")]
    InvalidComponentCount(usize),
    
    #[error("Empty component: {0} cannot be empty")]
    EmptyComponent(String),
    
    #[error("Invalid anchor: '{0}' must be 'var' or 'doc'")]
    InvalidAnchor(String),
    
    #[error("Invalid character in {0}: '{1}' - only alphanumeric, underscore, and hyphen allowed")]
    InvalidCharacter(String, String),
    
    #[error("Reserved name: '{0}' is a reserved system name")]
    ReservedName(String),
}

/// Validates and expands a context chain string into its canonical form.
/// 
/// # Arguments
/// * `input` - The context chain string to validate (e.g., "@project.workspace.var.keystore")
/// * `fallback_base` - Base name to use for CDCC (Cursor-Dependent Context Chain)
/// 
/// # Returns
/// * `Ok(ExpandedContext)` - Successfully validated and expanded context
/// * `Err(ContextValidationError)` - Validation error with detailed message
/// 
/// # Examples
/// ```
/// // FQCC (Fully Qualified Context Chain)
/// let result = validate_and_expand_context_chain("@work@website.api_keys.var.credentials", "home");
/// assert!(result.is_ok());
/// 
/// // CDCC (Cursor-Dependent Context Chain) 
/// let result = validate_and_expand_context_chain("@frontend.deployment.var.production", "work");
/// assert!(result.is_ok());
/// 
/// // Invalid context
/// let result = validate_and_expand_context_chain("invalid", "home");
/// assert!(result.is_err());
/// ```
pub fn validate_and_expand_context_chain(
    input: &str, 
    fallback_base: &str
) -> Result<ExpandedContext, ContextValidationError> {
    let mut logger = Stderr::new(&StderrConfig::from_env());
    
    logger.trace_fn("context_validator", &format!("validating context chain: '{}'", input));
    
    // Step 1: Basic validation
    if input.trim().is_empty() {
        logger.error("Context chain cannot be empty");
        return Err(ContextValidationError::EmptyChain);
    }
    
    let input = input.trim();
    
    // Step 2: Parse prefix mode
    let (prefix_mode, chain_body) = match input.chars().next() {
        Some('@') => {
            logger.trace_fn("context_validator", "detected persistent mode (@)");
            (ChainMode::Persistent, &input[1..])
        }
        Some('%') => {
            logger.trace_fn("context_validator", "detected ephemeral mode (%)");
            (ChainMode::Ephemeral, &input[1..])
        }
        Some('#') => {
            logger.trace_fn("context_validator", "detected action mode (#)");
            (ChainMode::Action, &input[1..])
        }
        _ => {
            logger.error(&format!("Invalid prefix in '{}' - must start with @, %, or #", input));
            return Err(ContextValidationError::InvalidPrefix);
        }
    };
    
    // Step 3: Determine FQCC vs CDCC and extract base
    let (base, rest, is_fqcc) = if chain_body.contains('@') {
        logger.trace_fn("context_validator", "detected FQCC format (contains @)");
        
        let parts: Vec<&str> = chain_body.splitn(2, '@').collect();
        if parts.len() != 2 {
            logger.error(&format!("Invalid FQCC format in '{}' - expected base@project.workspace.anchor.tail", input));
            return Err(ContextValidationError::InvalidFqccFormat);
        }
        
        let base_part = parts[0];
        let rest_part = parts[1];
        
        // Validate base name
        if base_part.is_empty() {
            logger.error("Base name cannot be empty in FQCC");
            return Err(ContextValidationError::EmptyBaseName);
        }
        
        if base_part.starts_with('@') || base_part.starts_with('%') || base_part.starts_with('#') {
            logger.error(&format!("Base name '{}' cannot have prefix characters", base_part));
            return Err(ContextValidationError::BaseWithPrefix(base_part.to_string()));
        }
        
        validate_component_name("base", base_part)?;
        
        (base_part.to_string(), rest_part, true)
    } else {
        logger.trace_fn("context_validator", &format!("detected CDCC format, using fallback base: '{}'", fallback_base));
        (fallback_base.to_string(), chain_body, false)
    };
    
    // Step 4: Parse the four main components
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() != 4 {
        logger.error(&format!(
            "Invalid component count in '{}' - expected 4 parts (project.workspace.anchor.tail), found {}", 
            input, parts.len()
        ));
        return Err(ContextValidationError::InvalidComponentCount(parts.len()));
    }
    
    let project = parts[0];
    let workspace = parts[1];
    let anchor_str = parts[2];
    let tail = parts[3];
    
    // Step 5: Validate each component
    validate_component_name("project", project)?;
    validate_component_name("workspace", workspace)?;
    validate_component_name("tail", tail)?;
    
    // Step 6: Parse and validate anchor
    let anchor = match anchor_str.to_lowercase().as_str() {
        "var" => {
            logger.trace_fn("context_validator", "detected variable context (var)");
            Anchor::Var
        }
        "doc" => {
            logger.trace_fn("context_validator", "detected document context (doc)");
            Anchor::Doc
        }
        _ => {
            logger.error(&format!("Invalid anchor '{}' - must be 'var' or 'doc'", anchor_str));
            return Err(ContextValidationError::InvalidAnchor(anchor_str.to_string()));
        }
    };
    
    // Step 7: Build canonical form
    let canonical_form = if is_fqcc {
        format!("{}@{}.{}.{}.{}", 
            match prefix_mode {
                ChainMode::Persistent => '@',
                ChainMode::Ephemeral => '%', 
                ChainMode::Action => '#',
            },
            base, project, workspace, 
            match anchor {
                Anchor::Var => "var",
                Anchor::Doc => "doc",
            },
            tail
        )
    } else {
        format!("{}{}.{}.{}.{}", 
            match prefix_mode {
                ChainMode::Persistent => '@',
                ChainMode::Ephemeral => '%',
                ChainMode::Action => '#',
            },
            project, workspace,
            match anchor {
                Anchor::Var => "var", 
                Anchor::Doc => "doc",
            },
            tail
        )
    };
    
    let expanded = ExpandedContext {
        base,
        project: project.to_string(),
        workspace: workspace.to_string(),
        anchor,
        tail: tail.to_string(),
        prefix_mode,
        is_fqcc,
        original_input: input.to_string(),
        canonical_form,
    };
    
    logger.trace_fn("context_validator", &format!("validation successful: '{}'", expanded.canonical_form));
    logger.info(&format!("✅ Context validated: {}", expanded.canonical_form));
    
    Ok(expanded)
}

/// Validates a component name (base, project, workspace, keystore, etc.)
fn validate_component_name(component_type: &str, name: &str) -> Result<(), ContextValidationError> {
    if name.is_empty() {
        return Err(ContextValidationError::EmptyComponent(component_type.to_string()));
    }
    
    // Check for invalid characters
    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            return Err(ContextValidationError::InvalidCharacter(
                component_type.to_string(),
                ch.to_string()
            ));
        }
    }
    
    // Check for reserved names (optional - add as needed)
    let reserved_names = ["null", "undefined", "nil", "void"];
    if reserved_names.contains(&name.to_lowercase().as_str()) {
        return Err(ContextValidationError::ReservedName(name.to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fqcc_validation() {
        let result = validate_and_expand_context_chain("@work@website.api_keys.var.credentials", "home");
        assert!(result.is_ok());
        
        let context = result.unwrap();
        assert_eq!(context.base, "work");
        assert_eq!(context.project, "website");
        assert_eq!(context.workspace, "api_keys");
        assert!(matches!(context.anchor, Anchor::Var));
        assert_eq!(context.tail, "credentials");
        assert!(context.is_fqcc);
        assert_eq!(context.canonical_form, "@work@website.api_keys.var.credentials");
    }
    
    #[test]
    fn test_cdcc_validation() {
        let result = validate_and_expand_context_chain("@frontend.deployment.var.production", "work");
        assert!(result.is_ok());
        
        let context = result.unwrap();
        assert_eq!(context.base, "work");
        assert_eq!(context.project, "frontend");
        assert_eq!(context.workspace, "deployment");
        assert!(matches!(context.anchor, Anchor::Var));
        assert_eq!(context.tail, "production");
        assert!(!context.is_fqcc);
    }
    
    #[test]
    fn test_ephemeral_mode() {
        let result = validate_and_expand_context_chain("%temp.test.doc.readme", "home");
        assert!(result.is_ok());
        
        let context = result.unwrap();
        assert!(matches!(context.prefix_mode, ChainMode::Ephemeral));
        assert!(matches!(context.anchor, Anchor::Doc));
    }
    
    #[test]
    fn test_invalid_prefix() {
        let result = validate_and_expand_context_chain("invalid.context", "home");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextValidationError::InvalidPrefix));
    }
    
    #[test]
    fn test_invalid_anchor() {
        let result = validate_and_expand_context_chain("@project.workspace.invalid.tail", "home");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextValidationError::InvalidAnchor(_)));
    }
    
    #[test]
    fn test_empty_component() {
        let result = validate_and_expand_context_chain("@project..var.keystore", "home");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextValidationError::EmptyComponent(_)));
    }
    
    #[test]
    fn test_invalid_component_count() {
        let result = validate_and_expand_context_chain("@project.workspace", "home");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextValidationError::InvalidComponentCount(_)));
    }
}
