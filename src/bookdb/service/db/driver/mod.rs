// src/db/mod.rs - Database module organization


mod core;
mod dbutils;
mod project;
mod workspace;
mod keystore;
mod docstore;

mod manager; //for managing base.sqlite3 files multi-base support

// sql constants are now dynamically generated from build.rs

// Re-export the main Database struct and related types
pub use core::Database;
pub use dbutils::{ExportItem, BaseStats};
pub use workspace::WorkspaceMetadata;

