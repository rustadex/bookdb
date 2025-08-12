// src/db/mod.rs - Database module organization

mod core;
mod base;
mod project;
mod workspace;
mod keystore;
mod docstore;

// Re-export the main Database struct and related types
pub use core::Database;
pub use base::{ExportItem, BaseStats};
pub use workspace::WorkspaceMetadata;
