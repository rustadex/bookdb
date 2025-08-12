// src/db.rs - Database module re-export


// not loading ./data because its picked up by build.rs

pub mod driver;



mod core;
mod dbutils; //export import etc
mod project;
mod workspace;
mod keystore;
mod docstore;

mod multibase;

// sql constants are now dynamically generated from build.rs

// Re-export the main Database struct and related types
pub use core::Database;
pub use utils::{ExportItem, BaseStats};
pub use workspace::WorkspaceMetadata;

