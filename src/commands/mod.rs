// src/commands/mod.rs - Command module structure
//
// Modular command architecture matching the existing codebase structure
// Each command is implemented in its own file for maintainability

pub mod getv;
pub mod setv;
pub mod ls;
pub mod export;
pub mod import;
pub mod migrate;
pub mod r#use;

// Document commands (for future Phase 3)
pub mod getd;
pub mod setd;

// Re-export command functions for easy access
pub use getv::execute as execute_getv;
pub use setv::execute as execute_setv;
pub use ls::execute as execute_ls;
pub use export::execute as execute_export;
pub use import::execute as execute_import;
pub use migrate::execute as execute_migrate;
pub use r#use::execute as execute_use;

// Document commands (placeholders for now)
pub use getd::execute as execute_getd;
pub use setd::execute as execute_setd;

/// Command dispatcher that routes to appropriate command modules
pub fn dispatch_command(
    command: &crate::cli::Command,
    context: &crate::context::ResolvedContext,
    database: &mut crate::db::Database,
) -> crate::error::Result<()> {
    use crate::cli::Command;
    
    match command {
        Command::Getv { key, .. } => {
            execute_getv(key, context, database)
        }
        Command::Setv { key_value, .. } => {
            execute_setv(key_value, context, database)
        }
        Command::Ls { target, .. } => {
            execute_ls(*target, context, database)
        }
        Command::Export { file_path, format, proj, ds, vs, doc, key, seg, .. } => {
            execute_export(
                file_path,
                format.as_deref(),
                (proj.as_deref(), ds.as_deref(), vs.as_deref(), doc.as_deref(), key.as_deref(), seg.as_deref()),
                context,
                database,
            )
        }
        Command::Import { file_path, mode, map_base, map_proj, map_ds, format, .. } => {
            execute_import(
                file_path,
                mode.as_deref().or(format.as_deref()),
                (map_base.as_deref(), map_proj.as_deref(), map_ds.as_deref()),
                context,
                database,
            )
        }
        Command::Migrate { dry_run, .. } => {
            execute_migrate(*dry_run, context, database)
        }
        Command::Use { context_str } => {
            execute_use(context_str, context, database)
        }
        Command::Getd { dik, .. } => {
            execute_getd(dik, context, database)
        }
        Command::Setd { dik_value, .. } => {
            execute_setd(dik_value, context, database)
        }
        Command::Install {} => {
            // Install is handled specially in main.rs before command dispatch
            Ok(())
        }
    }
}
