// src/db.rs - Database module re-export


pub mod driver;
pub mod data; // Contains the sql folders
pub mod sql;


// sql constants are now dynamically generated from build.rs

// Re-export the main Database struct and related types
pub use driver::Database;
pub use driver::dbutils::{ExportItem, BaseStats};
pub use driver::workspace::WorkspaceMetadata;

