// src/bookdb/service/ctx/ctx_types.rs
// Single source of truth for all context-related types and enums
// Implementations remain in their respective files
// 
// NOTE: This contains CURRENT/WORKING types for existing code
// V3 segment-based types will be in typesv3.rs

use serde::{Deserialize, Serialize};

// ============================================================================
// CORE CONTEXT CHAIN TYPES (CURRENT - WORKING)
// ============================================================================

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
    /// Tail: keystore name (for var) or document_key (for doc) - simple String
    pub tail: String,
    /// Chain prefix mode
    pub prefix_mode: ChainMode,
    /// Whether this is a Fully Qualified Context Chain (has explicit base@)
    pub is_fqcc: bool,
}

/// Resolved context with all fields populated for database operations
#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub base: String,
    pub project: String,
    pub workspace: String,
    pub anchor: Anchor,
    pub tail: String,
    pub prefix_mode: ChainMode,
}

/// Cursor state for CDCC resolution - tracks active context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorState {
    /// Current active base
    pub base_cursor: String,
    /// Current context within that base
    pub context_cursor: Option<ContextChain>,
}

// ============================================================================
// ENUMS (CURRENT - WORKING)
// ============================================================================

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

// ============================================================================
// DATABASE ID MAPPING TYPES (CURRENT - WORKING)
// ============================================================================

/// Maps context components to their database table columns and values
/// This is for efficient database queries and debugging
#[derive(Debug, Clone)]
pub enum ContextMap {
    /// Variable context with full column mapping
    /// Maps to: base.sqlite3 -> project_ns -> keyval_ns -> vars
    Variables { 
        base_name: String,        // Database file identifier
        pns_id: i64,              // project_ns.pns_id 
        kvns_id: i64,             // keyval_ns.kvns_id
        project: String,          // project_ns.pns_name
        workspace: String,        // keyval_ns.workspace_name
        keystore: String,         // keyval_ns.kvns_name
    },
    /// Document context with full column mapping  
    /// Maps to: base.sqlite3 -> project_ns -> doc_stores -> docs
    Document { 
        base_name: String,        // Database file identifier
        pns_id: i64,              // project_ns.pns_id
        ds_id: i64,               // doc_stores.ds_id (if applicable)
        project: String,          // project_ns.pns_name
        workspace: String,        // Workspace identifier
        document_key: String,     // Document identifier
    },
    /// Partial mapping - project level only
    PartialProject { 
        base_name: String,        // Database file identifier
        pns_id: i64,              // project_ns.pns_id
        project: String,          // project_ns.pns_name
    },
    /// Partial mapping - workspace level
    PartialWorkspace { 
        base_name: String,        // Database file identifier
        pns_id: i64,              // project_ns.pns_id
        project: String,          // project_ns.pns_name
        workspace: String,        // Workspace identifier
    },
}

/// Legacy ID chain resolution - maintains the chain of foreign key IDs
/// This follows the path of foreign keys to reach the target namespace
#[derive(Debug, Clone)]
pub enum ResolvedContextIds {
    /// Variable context ID chain: pns_id -> kvns_id -> var_id
    Variables { 
        pns_id: i64,        // project_ns.pns_id (root of chain)
        kvns_id: i64,       // keyval_ns.kvns_id (child of pns_id)
        // var_id would be determined at query time based on var_key
    },
    /// Document context ID chain: pns_id -> ds_id -> doc_id  
    Document { 
        pns_id: i64,        // project_ns.pns_id (root of chain)
        ds_id: i64,         // doc_stores.ds_id (child of pns_id)
        // doc_id would be determined at query time based on doc_key
    },
    /// Partial resolution - project level only
    PartialProject { 
        pns_id: i64,        // project_ns.pns_id
    },
    /// Partial resolution - keystore/workspace level for variables
    PartialKeystore { 
        pns_id: i64,        // project_ns.pns_id (parent)
        kvns_id: i64,       // keyval_ns.kvns_id (child)
    },
    /// Partial resolution - document store level
    PartialDocStore { 
        pns_id: i64,        // project_ns.pns_id (parent)
        ds_id: i64,         // doc_stores.ds_id (child)
    },
}

// ============================================================================
// UTILITY TYPES (CURRENT - WORKING)
// ============================================================================

/// Default resolver for CDCC and atomicity rules
pub struct DefaultResolver;

/// Context manager with stderr integration for rich user experience
pub struct ContextManager {
    // Note: Implementation details in context_manager.rs
}

// ============================================================================
// NOTES
// ============================================================================

/*
CURRENT WORKING TYPES:
- ContextChain: Uses simple String tail for backward compatibility
- ResolvedContext: Clean interface for existing database operations
- ContextMap: Full database column mappings
- ResolvedContextIds: Foreign key ID chains

FUTURE V3 TYPES (will be in typesv3.rs):
- Segment-based ContextChain variants
- VarContextChain/DocContextChain specialized types
- Tail enum with VarContextTail/DocContextTail
- Segments container types

DEPRECATED:
- models.rs types (renamed to _dep_models.rs)
*/

// ============================================================================
// ANALYSIS NOTES  
// ============================================================================

/*
CONTEXT CHAIN VARIANTS:

1. ContextChain - Generic chain using Tail enum for type safety
2. VarContextChain - Specialized for variable operations, keystore field
3. DocContextChain - Specialized for document operations, document_key field

TAIL TYPE SYSTEM:
- Tail::Keystore(String) - for variable contexts
- Tail::DocumentKey(String) - for document contexts
- Provides compile-time safety to prevent mixing contexts

RESOLVED CONTEXT TYPES:

1. ResolvedContext - Clean interface using Tail enum
2. ContextMap - Database column mappings with full context info
3. ResolvedContextIds - Foreign key ID chains for efficient queries

DATABASE ID PATTERNS:
- ContextMap: Full column mapping with string values (debugging friendly)
- ResolvedContextIds: Just the foreign key chain (query efficient)

UPDATED FOR SQLV2 SCHEMA:
- Uses project_ns.pns_id instead of old project.p_id
- Uses keyval_ns.kvns_id instead of old varstores.vs_id  
- Maps workspace_name correctly
- Supports both FQCC and CDCC resolution modes


*/
