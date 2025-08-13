// src/db.rs - Database module re-export


pub mod driver;
pub mod data; // Contains the sql folders
pub mod sql;


mod core;
mod dbutils; //export import etc
mod project;
mod workspace;
mod keystore;
mod docstore;

mod multibase;

// sql constants are now dynamically generated from build.rs

// Re-export the main Database struct and related types
pub use driver::Database;
pub use dbutils::{ExportItem, BaseStats};
pub use workspace::WorkspaceMetadata;

