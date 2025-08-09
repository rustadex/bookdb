mod cli;
mod context;
mod db;
mod error;
mod rdx_stderr;
mod sql;
mod commands;
mod config;
mod cursor;

use clap::Parser;
use rdx_stderr::{Level as LogLevel, set_level};

fn main() -> error::Result<()> {
    let cli = cli::Cli::parse();
    if cli.trace { set_level(LogLevel::Trace); } else if cli.debug { set_level(LogLevel::Debug); }
    log_info!("bookdb starting");

    let mode = match &cli.command {
        Some(cli::Command::Setv { .. })
        | Some(cli::Command::Setd { .. })
        | Some(cli::Command::Import { .. })
        | Some(cli::Command::Use { .. })
        | Some(cli::Command::Migrate { .. }) => context::ResolutionMode::GetOrCreate,
        _ => context::ResolutionMode::ReadOnly,
    };

    let paths = config::resolve_paths();
    config::ensure_dirs(&paths)?;

    if let Some(cli::Command::Install { force, db_path }) = &cli.command {
        let dbp = db_path.as_ref().map(|p| p.clone()).unwrap_or(paths.db_path.clone());
        if *force { if dbp.exists() { std::fs::remove_file(&dbp).ok(); } }
        let _ = db::Database::open_at(&dbp)?;
        println!("Installed DB at {}", dbp.display());
        return Ok(());
    }

    let database = db::Database::open_at(&paths.db_path)?;

    fn ctx_tokens(cmd: &Option<cli::Command>) -> Vec<String> {
        match cmd {
            Some(cli::Command::Getv { context_chain, .. }) |
            Some(cli::Command::Setv { context_chain, .. }) |
            Some(cli::Command::Getd { context_chain, .. }) |
            Some(cli::Command::Setd { context_chain, .. }) |
            Some(cli::Command::Ls   { context_chain, .. }) |
            Some(cli::Command::Import { context_chain, .. , .. }) |
            Some(cli::Command::Export { context_chain, .. , .. }) |
            Some(cli::Command::Migrate { context_chain, .. }) => context_chain.clone(),
            _ => Vec::new(),
        }
    }
    let tokens = ctx_tokens(&cli.command);

    let (cursor_base, cursor_chain) = cursor::read_cursor(&paths);

    let parsed = if !tokens.is_empty() {
        context::parse_chain_tokens(&tokens)
    } else if let Some(chain) = cursor_chain {
        let toks: Vec<String> = chain.split_whitespace().map(|s| s.to_string()).collect();
        context::parse_chain_tokens(&toks)
    } else {
        context::Parsed { ctx: context::Context::default(), had_anchor:false, persist_cursor:false }
    };

    let var_cmd = matches!(&cli.command, Some(cli::Command::Getv { .. } | cli::Command::Setv { .. }));
    let mut active_context = parsed.ctx.clone();
    if var_cmd && !parsed.had_anchor {
        active_context.active_namespace = context::Namespace::Variables;
    }

    if tokens.is_empty() {
        if let Some(base) = cursor_base {
            let parts: Vec<&str> = base.split('.').collect();
            if !parts.is_empty() {
                active_context.project_name = parts[0].to_string();
                if parts.len() > 1 { active_context.docstore_name = parts[1].to_string(); }
            }
        }
    }

    if parsed.persist_cursor && !tokens.is_empty() {
        let base_value = format!("{}.{}", active_context.project_name, active_context.docstore_name);
        let chain_value = tokens.join(" ");
        cursor::write_cursor(&paths, Some(&base_value), Some(&chain_value))?;
    }

    let resolved_ids = context::resolve_ids(&active_context, mode, &database)?;

    // Dispatch placeholder (match your existing project)
    match cli.command {
        Some(cli::Command::Getv { key, .. }) => println!("getv {} in {:?}", key, active_context),
        _ => { println!("{:#?}", active_context); }
    }

    Ok(())
}
