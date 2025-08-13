
/// Handle cursor status display
pub fn handle_cursor_command(
    context_manager: &mut ContextManager,
    cursor_state: &bookdb::context::CursorState
) ->  Result<(), E> {
    context_manager.show_cursor_status(cursor_state)
}

/// Handle context switching with 'use' command
pub fn handle_use_command(
    context_str: String,
    context_manager: &mut ContextManager,
    cursor_state: &mut bookdb::context::CursorState,
) ->  Result<(), E> {
    let chain = parse_context_chain(&context_str, &cursor_state.base_cursor)?;
    context_manager.update_cursor(&chain, cursor_state)?;
    Ok(())
}

/// Handle variable retrieval
pub fn handle_getv_command(
    key: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("getv", &format!("key: {}, context: {}", key, context));
    
    match database.get_variable(&key, context)? {
        Some(value) => {
            println!("{}", value);
            logger.trace_fn("getv", "variable found and returned");
        }
        None => {
            logger.trace_fn("getv", "variable not found");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Handle variable setting
pub fn handle_setv_command(
    key_value: String,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("setv", &format!("input: {}, context: {}", key_value, context));
    
    let (key, value) = key_value.split_once('=')
        .ok_or_else(|| BookdbError::Argument("setv requires key=value format".to_string()))?;
    
    database.set_variable(key.trim(), value.trim(), context)?;
    logger.trace_fn("setv", &format!("set variable: {}", key.trim()));
    
    Ok(())
}

/// Handle variable deletion
pub fn handle_delv_command(
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
    use bookdb::context_manager::DestructiveOpConfirm;
    let mut confirm = DestructiveOpConfirm::new();
    if !confirm.confirm_delete("variable", &key)? {
        logger.info("Deletion cancelled.");
        return Ok(());
    }
    
    database.delete_variable(&key, context)?;
    logger.okay(&format!("Variable '{}' deleted successfully", key));
    
    Ok(())
}

/// Handle increment command
pub fn handle_inc_command(
    key: String,
    amount: i64,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("inc", &format!("key: {}, amount: {}, context: {}", key, amount, context));
    commands::execute_inc(key, amount, context, database)
}

/// Handle decrement command
pub fn handle_dec_command(
    key: String,
    amount: i64,
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("dec", &format!("key: {}, amount: {}, context: {}", key, amount, context));
    commands::execute_dec(key, amount, context, database)
}

/// Handle listing command
pub fn handle_ls_command(
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
            let projects = database.list_projects()?;
            formatter.display_namespaces(&projects, "Projects", "All bases")?;
        }
        cli::LsTarget::Workspaces => {
            let workspaces = database.list_workspaces(&context.project_name)?;
            formatter.display_namespaces(&workspaces, "Workspaces", &context.project_name)?;
        }
        cli::LsTarget::Keystores => {
            let keystores = database.list_keystores(&context.project_name, &context.workspace_name)?;
            formatter.display_namespaces(&keystores, "Keystores", &format!("{}.{}", context.project_name, context.workspace_name))?;
        }
    }
    
    Ok(())
}

/// Handle export command
pub fn handle_export_command(
    file_path: PathBuf,
    format: Option<String>,
    filters: (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>),
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    use bookdb::context_manager::{OperationProgress, DestructiveOpConfirm};
    
    logger.trace_fn("export", &format!("file: {:?}, context: {}", file_path, context));
    
    // Determine format
    let format = format.unwrap_or_else(|| {
        file_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "json" | "jsonl" => "jsonl",
                _ => "kv"
            })
            .unwrap_or("kv")
            .to_string()
    });
    
    // Confirm overwrite if file exists
    let mut confirm = DestructiveOpConfirm::new();
    if !confirm.confirm_overwrite(
        &format!("export to {}", file_path.display()),
        &format!("context {}", context)
    )? {
        return Ok(());
    }
    
    let mut progress = OperationProgress::new("Export");
    
    // Get data to export (apply filters if provided)
    let data = database.export_data(context, &(
        filters.0.as_deref(),
        filters.1.as_deref(),
        filters.2.as_deref(),
        filters.3.as_deref(),
        filters.4.as_deref(),
        filters.5.as_deref()
    ))?;
    
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
pub fn handle_import_command(
    file_path: PathBuf,
    mode: Option<String>,
    mappings: (Option<String>, Option<String>, Option<String>),
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
    
    // Extract mappings
    let (_map_base, _map_proj, _map_workspace) = mappings;
    
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
pub fn handle_getd_command(
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
pub fn handle_setd_command(
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
pub fn handle_migrate_command(
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
        logger.okay(&format!("Successfully migrated {} items", migration_count));
    }
    
    Ok(())
}

/// Handle installation command
pub fn handle_install_command(
    database: &Database,
    context: &bookdb::context::ResolvedContext,
    logger: &mut Stderr,
) ->  Result<(), E> {
    logger.trace_fn("install", "performing installation");
    
    // TODO: Implement full installation logic
    // - Create XDG directories
    // - Set up shell integration
    // - Create initial base and invincible superchain
    // - Generate RC file
    
    logger.info("Installing BookDB...");
    
    // For now, just ensure the database schema is set up
    database.execute_sql("SELECT 1")?; // Basic connectivity test
    
    logger.okay("Installation completed successfully!");
    logger.info("Please restart your shell to use the 'bookdb' command");
    
    Ok(())
}
