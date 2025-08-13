// src/db/mod.rs - Database module organization


pub mod core;
pub mod dbutils;
pub mod project;
pub mod workspace;
pub mod keystore;
pub mod docstore;
pub mod multibase;

pub mod manager; //for managing base.sqlite3 files multi-base support

// sql constants are now dynamically generated from build.rs

// Re-export the main Database struct and related types
pub use core::Database;
pub use dbutils::{ExportItem, BaseStats};
pub use workspace::WorkspaceMetadata;

// Modules are already declared as pub mod above, so they're accessible directly
