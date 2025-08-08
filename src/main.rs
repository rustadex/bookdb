// src/main.rs
mod cli;
mod commands;
mod config;
mod context;
mod db;
mod error;
mod models;
mod sql;
use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use config::Config;
use context::{ContextResolver, ResolutionMode};
use db::Database;
use error::BookdbError;
use log::info;
fn main() -> Result<()> {
  env_logger::init();


  // --- 1. Load Configuration from Environment ---
  let config = Config::from_env()?;

  // --- 2. Initialization & Correct Context-Aware Argument Parsing ---
  let xdg_dirs = xdg::BaseDirectories::with_prefix("bookdb-rs")
      .map_err(|e| BookdbError::XdgPath(e.to_string()))?;

  let cursor_path = xdg_dirs.place_config_file("context.json")?;
  let mut raw_args: Vec<String> = std::env::args().collect();

  let mut active_context = context::load_or_create_context(&xdg_dirs)?;

  if raw_args.len() > 1 {
      if let Some(last_arg) = raw_args.last() {
          if context::parse_context_string(last_arg).is_ok() {
              let context_str = raw_args.pop().unwrap();
              info!("Using command-line override context: {}", context_str);
              active_context = context::parse_context_string(&context_str)?;
          }
      }
  }

  let cli_matches = cli::Cli::command().get_matches_from(raw_args);
  let cli = cli::Cli::from_arg_matches(&cli_matches)?;

  let command = match cli.command {
      Some(cmd) => cmd,
      None => {
          // Default action: if no command is given, show the cursor and exit.
          println!("{:#?}", active_context);
          return Ok(());
      }
  };

  // --- 3. Determine Resolution Mode based on Command ---
  let mode = match &command {
      cli::Command::Setv { .. } | cli::Command::Setd { .. } | cli::Command::Import { .. } | cli::Command::Use { .. } => {
          ResolutionMode::GetOrCreate
      }
      _ => ResolutionMode::ReadOnly,
  };

  // --- 4. Setup Database and Resolver ---
  let db_path = xdg_dirs.place_data_file(format!("{}.sqlite", active_context.base_name))?;
  info!("Using database: {}", db_path.display());
  let database = Database::new(&db_path)?;

  if config.safe_mode && mode == ResolutionMode::GetOrCreate {
      let backup_file = xdg_dirs.place_cache_file(format!("backup-{}.sqlite", chrono::Utc::now().timestamp()))?;
      info!("SAFE_MODE: Backing up database to {}", backup_file.display());
      database.backup_db(&backup_file)?;
  }

  // --- 5. Resolve Context and Dispatch Command ---
  let resolver = ContextResolver::new(&database);

  match command {
      cli::Command::Ls { target: cli::LsTarget::Projects } => {
          commands::ls::execute_projects(&database)?
      }
      cli::Command::Use { context_str } => commands::r#use::execute(&context_str, &cursor_path)?,
      cli::Command::Cursor => println!("{:#?}", active_context),
      
      _ => {
          let resolved_ids = resolver.resolve(&active_context, mode)?;
          info!("Resolved IDs for operation: {:?}", resolved_ids);
          
          match command {
              cli::Command::Getv { key } => commands::getv::execute(&key, &database, resolved_ids)?,
              cli::Command::Setv { key_value } => commands::setv::execute(&key_value, &database, resolved_ids)?,
              cli::Command::Getd { dik } => commands::getd::execute(&dik, &database, resolved_ids)?,
              cli::Command::Setd { dik_value } => commands::setd::execute(&dik_value, &database, resolved_ids)?,
              cli::Command::Ls { target } => commands::ls::execute(target, &database, resolved_ids)?,
              cli::Command::Import { file_path } => commands::import::execute(&file_path, &database, resolved_ids)?,
              cli::Command::Export { file_path } => commands::export::execute(&file_path, &database, resolved_ids)?,
              _ => unreachable!(),
          }
      }
  }

  Ok(())
}
