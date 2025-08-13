// src/main.rs - Complete main entry point with all command handlers



use stderr::{Stderr, StderrConfig};

//use crate::admin::install::{InstallationManager, require_installation_or_install};

use crate::error::{Result, BookdbError};
use crate::bookdb::service::db::Database;
use crate::ctx::{ContextManager, parse_context_chain};



pub fn load_global_context( args: &mut Vec<T>, ) -> (){

  // Handle global context if provided
  if let Some(global_context) = args.context {
      let chain = parse_context_chain(&global_context, &cursor_state.base_cursor)?;
      if args.persist {
          context_manager.update_cursor(&chain, &mut cursor_state)?;
      }
      cursor_state = bookdb::context::CursorState {
          base_cursor: chain.base_name.clone(),
          context_cursor: chain.clone(),
      };
  }

}

pub fn open_database( context_manager: &mut ContextManager, 
                      cursor_state: &mut CursorState ) -> &Database 
{
  // Open database
  let database_path = context_manager.get_database_path(&cursor_state.base_cursor);
  let database = if matches!(args.command, Some(cli::Command::Install {})) {
      // Installation - create database if needed
      Database::create_or_open(&database_path)?
  } else {
      // Normal operation - database must exist
      Database::open(&database_path)?
  };

  // error ?
  database
}

pub fn resolve_context_chain( args: &mut Vec<T>, context_manager: &mut ContextManager, 
                              cursor_state: &mut CursorState ) -> &Database 
{
  // Get context from command or use cursor
  let context_chain = get_context_from_command(&args.command)
      .map(|ctx| parse_context_chain(&ctx, &cursor_state.base_cursor))
      .transpose()?
      .unwrap_or_else(|| cursor_state.context_cursor.clone());
  
  // Resolve context for database operations
  let resolved_context = context_manager.resolve_context(&context_chain)?;

  resolved_context  

}



pub fn dispatch_router(args: Vec<T>, database: &Database, context_chain: &str) ->  Result<(), E> {

  let mut logger = Stderr::new();

  // Route commands
  match args.command {
    Some(cli::Command::Getv { key, .. }) => {
        handle_getv_command(key, &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Setv { key_value, .. }) => {
        handle_setv_command(key_value, &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Delv { key, .. }) => {
        handle_delv_command(key, &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Inc { key, amount, .. }) => {
        handle_inc_command(key, amount, &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Dec { key, amount, .. }) => {
        handle_dec_command(key, amount, &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Ls { target, .. }) => {
        handle_ls_command(target, &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Export { file_path, format, proj, workspace, keystore, doc, key, seg, .. }) => {
        handle_export_command(file_path, format, (proj, workspace, keystore, doc, key, seg), &database, &resolved_context, &mut logger)
    }
    Some(cli::Command::Import { file_path, mode, map_base, map_proj, map_workspace, .. }) => {
        handle_import_command(file_path, mode, (map_base, map_proj, map_workspace), &database, &resolved_context, &mut logger)
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
    Some(cli::Command::Use { context_str }) => {
        handle_use_command(context_str, &mut context_manager, &mut cursor_state)
    }
    Some(cli::Command::Install {}) => {
        handle_install_command(&database, &resolved_context, &mut logger)
    }
    None => {
        // No command specified, show cursor status
        handle_cursor_command(&mut context_manager, &cursor_state)
    }
  }
  
}



fn dispatch() ->  Result<(), E> {



    let mut logger = Stderr::new();
    
    let args = cli::Cli::parse();
    
    // Load configuration
    let config = Config::load().map_err(|e| {
        logger.error(&format!("Failed to load configuration: {}", e));
        e
    })?;



    // Initialize context manager
    let mut context_manager = ContextManager::new();
    let mut cursor_state = context_manager.load_cursor_state()?;


    load_global_context( &args, &context_manager, &cursor_state);

    open_database(&context_manager, &cursor_state);

    resolve_context_chain(&args, &context_manager, &cursor_state); 

    dispatch_router()

}

