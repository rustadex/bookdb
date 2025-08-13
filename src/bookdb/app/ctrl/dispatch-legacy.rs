// src/main.rs - Updated with Phase 1 context system and installation guard
//
// CRITICAL CHANGES:
// 1. Installation guard blocks usage until 'bookdb install'
// 2. New CONCEPTS.md-compliant context system
// 3. Stderr integration with context banners
// 4. Proper error handling and user feedback

use clap::Parser;
use bookdb::{
    cli::{self, Cli},
    config::Config,
    context::{parse_context_chain, DefaultResolver, ContextChain, ChainMode},
    context_manager::{ContextManager, DestructiveOpConfirm},
    installation::{InstallationManager, require_installation_or_install},
    db::Database,
    error::{Result, BookdbError},
    rdx::stderr::{Stderr, StderrConfig},
};

fn main() ->  Result<(), E> {
    // Initialize stderr logging
    let mut logger = Stderr::new();
    logger.trace_fn("main", "BookDB starting");
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Load configuration
    let config = Config::load().map_err(|e| {
        logger.error(&format!("Failed to load configuration: {}", e));
        e
    })?;
    
    // Check if this is the install command
    let is_install_command = matches!(cli.command, Some(cli::Command::Install {}));
    
    // CRITICAL: Check installation status before proceeding
    if let Err(e) = require_installation_or_install(&config, is_install_command) {
        match e {
            BookdbError::NotInstalled(_) => {
                // Installation guard already showed user-friendly message
                std::process::exit(1);
            }
            _ => return Err(e),
        }
    }
    
    // Handle install command specially
    if is_install_command {
        return handle_install_command(config);
    }
    
    // For all other commands, proceed with normal operation
    execute_command(cli, config, logger)
}

/// Handle the install command
fn handle_install_command(config: Config) ->  Result<(), E> {
    let mut installer = InstallationManager::new(config);
    installer.install()
}

/// Execute a regular BookDB command
fn execute_command(cli: Cli, config: Config, mut logger: Stderr) ->  Result<(), E> {
    logger.trace_fn("main", "executing command");
    
    // Initialize context manager
    let mut context_manager = ContextManager::new(config.clone());
    
    // Load current cursor state
    let mut cursor_state = context_manager.load_cursor_state()?;
    
    // Parse context chain if provided
    let resolved_context = if let Some(context_str) = get_context_from_command(&cli.command) {
        logger.trace_fn("main", &format!("parsing context: {}", context_str));
        
        let chain = parse_context_chain(&context_str, &cursor_state.base_cursor)?;
        
        // Update cursor if this is a persistent operation
        match chain.prefix_mode {
            ChainMode::Persistent => {
                context_manager.update_cursor(&chain, &mut cursor_state)?;
            }
            ChainMode::Ephemeral => {
                // Show ephemeral context banner but don't update cursor
                context_manager.show_context_banner(&chain)?;
            }
            ChainMode::Action => {
                // Future: handle action mode
                logger.trace_fn("main", "action mode not yet implemented");
            }
        }
        
        // Resolve to full context
        DefaultResolver::new().resolve_cdcc(&chain, &cursor_state)
    } else {
        // Use cursor defaults
        if let Some(ref context_chain) = cursor_state.context_cursor {
            DefaultResolver::new().resolve_cdcc(context_chain, &cursor_state)
        } else {
            // No context set, use invincible superchain
            let superchain = DefaultResolver::create_invincible_superchain(&cursor_state.base_cursor);
            DefaultResolver::new().resolve_cdcc(&superchain, &cursor_state)
        }
    };
    
    logger.trace_fn("main", &format!("resolved context: {}", resolved_context));
    
    // Open database for the resolved base
    let database = Database::open(&config.get_base_path(&resolved_context.base))?;
    
    // Execute the specific command
    match cli.command {
        Some(cli::Command::Cursor {}) => {
            handle_cursor_command(&mut context_manager, &cursor_state)
        }
        Some(cli::Command::Use { context_str }) => {
            handle_use_command(context_str, &mut context_manager, &mut cursor_state)
        }
        Some(cli::Command::Getv { key, .. }) => {
            handle_getv_command(key, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Setv { key_value, .. }) => {
            handle_setv_command(key_value, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Delv { key, .. }) => {
            handle_delv_command(key, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Ls { target, .. }) => {
            handle_ls_command(target, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Export { file_path, format, proj, ds, vs, doc, key, seg, .. }) => {
            handle_export_command(file_path, format, (proj, ds, vs, doc, key, seg), &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Import { file_path, mode, map_base, map_proj, map_ds, .. }) => {
            handle_import_command(file_path, mode, (map_base, map_proj, map_ds), &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Getd { dik, .. }) => {
            handle_getd_command(dik, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Setd { dik_value, .. }) => {
            handle_setd_command(dik_value, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Migrate { dry_run, .. }) => {
            handle_migrate_command(dry_run, &database, &resolved_context, &mut logger)
        }
        Some(cli::Command::Install {}) => {
            // Already handled above
            Ok(())
        }
        None => {
            // No command specified, show cursor status
            handle_cursor_command(&mut context_manager, &cursor_state)
        }
    }
}

/// Extract context string from command if present
fn get_context_from_command(command: &Option<cli::Command>) -> Option<String> {
    match command {
        Some(cli::Command::Getv { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Setv { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Delv { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Getd { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Setd { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Ls { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Import { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Export { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Migrate { context_chain, .. }) => context_chain.clone(),
        _ => None,
    }
}

/// Handle cursor status display
fn handle_cursor_command(
    context_manager: &mut ContextManager, 
    cursor_state: &bookdb::context::CursorState
) ->  Result<(), E> {
    context_manager.show_cursor_status(cursor_state)
}

/// Handle context switching with 'use' command
fn handle_use_command(
    context_str: String,
    context_manager: &mut ContextManager,
    cursor_state: &mut bookdb::context::CursorState,
) ->  Result<(), E> {
    let chain = parse_context_chain(&context_str, &cursor_state.base_cursor)?;
    context_manager.update_cursor(&chain, cursor_state)?;
    Ok(())
}

/// Handle variable retrieval
fn handle_getv_command(
    key: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("getv", &format!("key: {}, context: {}", key, context));
    
    match database.get_variable(&key, context)? {
        Some(value) => {
            println!("{}", value);
            logger.trace_fn("getv", "value found and returned");
        }
        None => {
            logger.trace_fn("getv", "key not found");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Handle variable setting
fn handle_setv_command(
    key_value: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("setv", &format!("input: {}, context: {}", key_value, context));
    
    let (key, value) = key_value.split_once('=')
        .ok_or_else(|| BookdbError::Argument("setv requires key=value format".to_string()))?;
    
    database.set_variable(key.trim(), value.trim(), context)?;
    logger.trace_fn("setv", &format!("set {}={}", key.trim(), value.trim()));
    
    Ok(())
}

/// Handle variable deletion with confirmation
fn handle_delv_command(
    key: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("delv", &format!("key: {}, context: {}", key, context));
    
    // Check if key exists first
    if database.get_variable(&key, context)?.is_none() {
        logger.warn(&format!("Key '{}' not found in context {}", key, context));
        return Ok(());
    }
    
    // Confirm deletion for safety
    let mut confirm = DestructiveOpConfirm::new();
    if !confirm.confirm_delete("variable", &key)? {
        logger.info("Deletion cancelled.");
        return Ok(());
    }
    
    database.delete_variable(&key, context)?;
    logger.okay(&format!("Variable '{}' deleted successfully", key));
    
    Ok(())
}

/// Handle ls command with rich table formatting
fn handle_ls_command(
    target: cli::LsTarget,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    use bookdb::context_manager::LsTableFormatter;
    
    logger.trace_fn("ls", &format!("target: {:?}, context: {}", target, context));
    
    let mut formatter = LsTableFormatter::new();
    
    match target {
        cli::LsTarget::Keys => {
            let variables = database.list_variables(context)?;
            formatter.display_variables(&variables, &format!("{}", context))?;
        }
        cli::LsTarget::Docs => {
            let documents = database.list_documents(context)?;
            formatter.display_namespaces(&documents, "Documents", &format!("{}", context))?;
        }
        cli::LsTarget::Projects => {
            let projects = database.list_projects(&context.base)?;
            formatter.display_namespaces(&projects, "Projects", &context.base)?;
        }
        cli::LsTarget::Docstores => {
            let workspaces = database.list_workspaces(&context.base, &context.project)?;
            formatter.display_namespaces(&workspaces, "Workspaces", &format!("{}.{}", context.base, context.project))?;
        }
        cli::LsTarget::Varstores => {
            let keystores = database.list_keystores(&context.base, &context.project, &context.workspace)?;
            formatter.display_namespaces(&keystores, "Keystores", &format!("{}.{}.{}", context.base, context.project, context.workspace))?;
        }
    }
    
    Ok(())
}

/// Handle export command with progress tracking
fn handle_export_command(
    file_path: std::path::PathBuf,
    format: Option<String>,
    filters: (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>),
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    use bookdb::context_manager::OperationProgress;
    
    logger.trace_fn("export", &format!("file: {:?}, context: {}", file_path, context));
    
    let format = format.unwrap_or_else(|| {
        // Auto-detect format from file extension
        file_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "json" | "jsonl" => "jsonl",
                _ => "kv",
            })
            .unwrap_or("kv")
            .to_string()
    });
    
    logger.info(&format!("Exporting to {} in {} format", file_path.display(), format));
    
    let mut progress = OperationProgress::new("Export");
    
    // Get data to export
    let data = database.export_data(context, &filters)?;
    progress.set_total(data.len());
    
    // Write to file with progress tracking
    let mut output = std::fs::File::create(&file_path)?;
    
    for (i, item) in data.iter().enumerate() {
        progress.increment(&format!("item {}", i + 1))?;
        
        match format.as_str() {
            "jsonl" => {
                use std::io::Write;
                writeln!(output, "{}", serde_json::to_string(item)?)?;
            }
            "kv" => {
                use std::io::Write;
                writeln!(output, "{}={}", item.key, item.value)?;
            }
            _ => return Err(BookdbError::Argument(format!("Unsupported export format: {}", format))),
        }
    }
    
    progress.complete()?;
    logger.okay(&format!("Successfully exported {} items to {}", data.len(), file_path.display()));
    
    Ok(())
}

/// Handle import command with progress tracking and confirmation
fn handle_import_command(
    file_path: std::path::PathBuf,
    mode: Option<String>,
    mapping: (Option<String>, Option<String>, Option<String>),
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    use bookdb::context_manager::{OperationProgress, DestructiveOpConfirm};
    
    logger.trace_fn("import", &format!("file: {:?}, context: {}", file_path, context));
    
    if !file_path.exists() {
        return Err(BookdbError::Argument(format!("File not found: {}", file_path.display())));
    }
    
    let mode = mode.unwrap_or_else(|| "merge".to_string());
    
    // Confirm potentially destructive operation
    let mut confirm = DestructiveOpConfirm::new();
    if !confirm.confirm_overwrite(
        &format!("import from {}", file_path.display()),
        &format!("context {}", context)
    )? {
        return Ok(());
    }
    
    let mut progress = OperationProgress::new("Import");
    
    // Read and process file
    let content = std::fs::read_to_string(&file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    progress.set_total(lines.len());
    
    let mut imported_count = 0;
    
    for (i, line) in lines.iter().enumerate() {
        progress.increment(&format!("line {}", i + 1))?;
        
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            
            if !key.is_empty() {
                database.set_variable(key, value, context)?;
                imported_count += 1;
            }
        }
    }
    
    progress.complete()?;
    logger.okay(&format!("Successfully imported {} variables from {}", imported_count, file_path.display()));
    
    Ok(())
}

/// Handle document retrieval
fn handle_getd_command(
    dik: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("getd", &format!("dik: {}, context: {}", dik, context));
    
    match database.get_document(&dik, context)? {
        Some(content) => {
            println!("{}", content);
            logger.trace_fn("getd", "document found and returned");
        }
        None => {
            logger.trace_fn("getd", "document not found");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Handle document setting
fn handle_setd_command(
    dik_value: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("setd", &format!("input: {}, context: {}", dik_value, context));
    
    let (dik, content) = dik_value.split_once('=')
        .ok_or_else(|| BookdbError::Argument("setd requires dik=content format".to_string()))?;
    
    database.set_document(dik.trim(), content.trim(), context)?;
    logger.trace_fn("setd", &format!("set document: {}", dik.trim()));
    
    Ok(())
}

/// Handle migration command
fn handle_migrate_command(
    dry_run: bool,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("migrate", &format!("dry_run: {}, context: {}", dry_run, context));
    
    if dry_run {
        logger.info("DRY RUN: No changes will be made");
    }
    
    let migration_count = database.migrate_legacy_data(context, dry_run)?;
    
    if dry_run {
        logger.info(&format!("Migration preview: {} items would be migrated", migration_count));
    } else {
        logger.okay(&format!("Migration completed: {} items migrated", migration_count));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_context_extraction() {
        let cmd = cli::Command::Getv { 
            key: "test".to_string(), 
            context_chain: Some("@base@proj.workspace.var.keystore".to_string()) 
        };
        
        let context = get_context_from_command(&Some(cmd));
        assert_eq!(context, Some("@base@proj.workspace.var.keystore".to_string()));
    }
    
    #[test]
    fn test_main_error_handling() {
        // This test would require mocking, but demonstrates the structure
        // In practice, integration tests would verify full command execution
    }
}
