// src/models.rs - Updated with consistent BOOKDB_CONCEPTS.md terminology

// todo: is this even being used anywhere?

use serde::{Deserialize, Serialize};

/// Represents the active context for database operations.
/// This will be read from/written to the context cursor file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Context {
    pub base_name: String,
    pub project_name: String,
    pub workspace_name: String,        // FIXED: was docstore_name
    pub active_namespace: Namespace,
}

/// Defines whether the current context points to a keystore or a document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Namespace {
    /// For contexts like: base@project.workspace.var.keystore
    Variables { keystore_name: String },   // FIXED: was varstore_name
    /// For contexts like: base@project.workspace.doc
    Document,
}

impl Default for Context {
    /// Provides a sensible default context for first-time runs.
    /// Matches BOOKDB_CONCEPTS.md invincible superchain: ROOT.GLOBAL.VAR.MAIN
    fn default() -> Self {
        Context {
            base_name: "home".to_string(),
            project_name: "ROOT".to_string(),           // FIXED: was "GLOBAL" 
            workspace_name: "GLOBAL".to_string(),       // FIXED: was docstore_name: "main"
            active_namespace: Namespace::Variables {
                keystore_name: "MAIN".to_string(),      // FIXED: was varstore_name: "default"
            },
        }
    }
}
