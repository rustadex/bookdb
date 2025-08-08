mod cli;
mod context;
mod db;
mod error;
mod rdx_stderr;
mod sql;
mod commands;

use clap::Parser;
use rdx_stderr::Level as LogLevel;
use rdx_stderr::set_level;

fn main() -> error::Result<()> {
    // 1) CLI parse
    let cli = cli::Cli::parse();

    // 2) stderr log setup
    if cli.trace { set_level(LogLevel::Trace); } else if cli.debug { set_level(LogLevel::Debug); }
    log_info!("bookdb starting");

    // 3) Determine mode
    let command = cli.command.clone();
    let mode = match command {
        Some(cli::Command::Setv { .. })
        | Some(cli::Command::Setd { .. })
        | Some(cli::Command::Import { .. })
        | Some(cli::Command::Use { .. })
        | Some(cli::Command::Migrate { .. }) => context::ResolutionMode::GetOrCreate,
        _ => context::ResolutionMode::ReadOnly,
    };

    // 4) Open DB
    let database = db::Database::open_default()?;

    // 5) Resolve context (for this clean build we default to GLOBAL.main)
    let active_context = context::Context::default();
    let resolved_ids = context::resolve_ids(&active_context, mode, &database)?;

    // 6) Dispatch
    match cli.command {
        Some(cli::Command::Getv { key }) => commands::getv::execute(&key, &database, resolved_ids)?,
        Some(cli::Command::Setv { key_value }) => commands::setv::execute(&key_value, &database, resolved_ids)?,
        Some(cli::Command::Getd { dik }) => commands::getd::execute(&dik, &database, resolved_ids)?,
        Some(cli::Command::Setd { dik_value }) => commands::setd::execute(&dik_value, &database, resolved_ids)?,
        Some(cli::Command::Ls { target }) => commands::ls::execute(target, &database, resolved_ids)?,
        Some(cli::Command::Export { file_path, format, proj, ds, vs, doc, key, seg }) => {
            commands::export::execute(&file_path, format, (proj,ds,vs,doc,key,seg), &database, resolved_ids)?
        }
        Some(cli::Command::Import { file_path, mode, map_base, map_proj, map_ds, format: _ }) => {
            commands::import::execute(&file_path, &mode, &map_base, &map_proj, &map_ds, &database, resolved_ids)?
        }
        Some(cli::Command::Migrate { dry_run }) => commands::migrate::execute(dry_run, &database, resolved_ids)?,
        Some(cli::Command::Use { context_str }) => commands::r#use::execute(&context_str)?,
        None => { println!("{:#?}", active_context); }
    }

    Ok(())
}
