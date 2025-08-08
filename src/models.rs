// src/models.rs

use serde::{Deserialize, Serialize};

/// Represents the active context for database operations.
/// This will be read from/written to the context cursor file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Context {
    pub base_name: String,
    pub project_name: String,
    pub docstore_name: String,
    pub active_namespace: Namespace,
}

/// Defines whether the current context points to a variable store or a document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Namespace {
    /// For contexts like: base@proj.docstore.var.varstore
    Variables { varstore_name: String },
    /// For contexts like: base@proj.docstore.doc
    Document,
}

impl Default for Context {
    /// Provides a sensible default context for first-time runs.
    fn default() -> Self {
        Context {
            base_name: "home".to_string(),
            project_name: "GLOBAL".to_string(),
            docstore_name: "main".to_string(),
            active_namespace: Namespace::Variables {
                varstore_name: "default".to_string(),
            },
        }
    }
}
