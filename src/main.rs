mod cli;
mod context;
mod db;
mod error;
mod rdx_stderr;
mod sql;
mod commands;
mod config;

use clap::Parser;
use rdx_stderr::Level as LogLevel;
use rdx_stderr::set_level;
use error::{Result, BookdbError};
use std::path::PathBuf;

// --- helpers ---------------------------------------------------------------

fn read_cursor_chain() -> Result<String> {
    let paths = config::resolve_paths();
    if let Ok(s) = std::fs::read_to_string(&paths.cursor_chain_path) {
        let t = s.trim().to_string();
        if !t.is_empty() { return Ok(t); }
    }
    Err(BookdbError::ContextParse("no context chain provided and cursor.chain missing".into()))
}

fn read_cursor_base_or_default() -> Result<PathBuf> {
    let paths = config::resolve_paths();
    Ok(paths._base_db_path.clone())
}

fn ensure_dirs_for_base(p: &PathBuf) -> Result<()> {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

struct ParsedAndDb {
    parsed: context::Parsed,
}

// Determine db path and parsed context from maybe-chain
fn parse_and_open_base(maybe_chain: Option<&str>) -> Result<(ParsedAndDb, db::Database)> {
    // Base fallback from env/cursor/default
    let base_fallback_abs = read_cursor_base_or_default()?;
    let base_fallback_str = base_fallback_abs.to_string_lossy().to_string();

    // Parse
    let parsed = if let Some(chain) = maybe_chain {
        context::parse_strict_fqcc(chain, &base_fallback_str)?
    } else {
        let chain = read_cursor_chain()?;
        context::parse_strict_fqcc(&chain, &base_fallback_str)?
    };

    // Choose DB path: if chain had explicit base, ctx.base_db_abs has it; else fallback
    let db_path = if parsed.had_explicit_base {
        PathBuf::from(&parsed.ctx.base_db_abs)
    } else {
        base_fallback_abs
    };

    ensure_dirs_for_base(&db_path)?;
    let database = db::Database::open_at(&db_path)?;

    Ok((ParsedAndDb { parsed }, database))
}

// Normalize Option<String> to owned String or Vec<String>
fn opt_str(opt: &Option<String>, default: &str) -> String {
    opt.as_deref().unwrap_or(default).to_string()
}
fn opt_list(opt: &Option<String>) -> Vec<String> {
    opt.as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_else(|| Vec::<String>::new())
}

// --- main ------------------------------------------------------------------

fn main() -> Result<()> {
    // 1) CLI parse
    let cli = cli::Cli::parse();

    // 2) stderr log setup
    if cli.trace {
        set_level(LogLevel::Trace);
    } else if cli.debug {
        set_level(LogLevel::Debug);
    }
    log_info!("bookdb starting");

    // 3) Calculate mode (affects creation behavior during resolution)
    let mode = match &cli.command {
        Some(cli::Command::Setv { .. })
        | Some(cli::Command::Setd { .. })
        | Some(cli::Command::Import { .. })
        | Some(cli::Command::Use { .. })
        | Some(cli::Command::Migrate { .. })
        | Some(cli::Command::Install { .. }) => context::ResolutionMode::ReadOnly,
        _ => context::ResolutionMode::ReadOnly,
    };



    // 4) Pull context chain (always LAST positional)
    let maybe_chain_str: Option<&str> = match &cli.command {
        Some(cli::Command::Getv { context_chain, .. }) |
        Some(cli::Command::Setv { context_chain, .. }) |
        Some(cli::Command::Getd { context_chain, .. }) |
        Some(cli::Command::Setd { context_chain, .. }) |
        Some(cli::Command::Ls   { context_chain, .. }) |
        Some(cli::Command::Import { context_chain, .. }) |
        Some(cli::Command::Export { context_chain, .. }) |
        Some(cli::Command::Migrate { context_chain, .. }) => context_chain.as_deref(),
        _ => None,
    };

    // 5) Special-case Install (no context needed)
    if let Some(cli::Command::Install {}) = &cli.command {
        let paths = config::resolve_paths();
        config::ensure_dirs(&paths)?;
        let home_db = config::default_home_db_path(&paths.data_dir);
        ensure_dirs_for_base(&home_db)?;
        let db = db::Database::open_at(&home_db)?;
        db.bootstrap_schema()?;
        db.seed_home_invincibles()?;
        db.mark_installed()?;
        println!("Installed: {}", home_db.display());
        return Ok(());
    }

    // 6) Parse chain + open base DB
    let (parsed_db, database) = parse_and_open_base(maybe_chain_str)?;
    let parsed = parsed_db.parsed;

    // Enforce "not installed yet" behavior (no AUTO install)
    if !database.is_installed()? {
        return Err(BookdbError::Argument("bookdb is not installed for this base; run `bookdb install`".into()));
    }

    // 7) Resolve context IDs (requires the tail segment)
    let ids = context::resolve_ids(&parsed.ctx, &parsed.tail, mode, &database)?;

    // 8) Dispatch
    match cli.command {
        Some(cli::Command::Getv { key, .. }) => {
            commands::getv::execute(&key, &database, ids)?
        }
        Some(cli::Command::Setv { key_value, .. }) => {
            commands::setv::execute(&key_value, &database, ids)?
        }
        Some(cli::Command::Getd { dik, .. }) => {
            commands::getd::execute(&dik, &database, ids)?
        }
        Some(cli::Command::Setd { dik_value, .. }) => {
            commands::setd::execute(&dik_value, &database, ids)?
        }
        Some(cli::Command::Ls { target, .. }) => {
            commands::ls::execute(target, &database, ids)?
        }
        Some(cli::Command::Export { file_path, format, proj, ds, vs, doc, key, seg, .. }) => {
            commands::export::execute(
                &file_path,
                format,
                (proj, ds, vs, doc, key, seg),
                &database,
                ids
            )?
        }
        Some(cli::Command::Import { file_path, mode, map_base, map_proj, map_ds, .. }) => {
            let mode_s  = opt_str(&mode, "auto");
            let map_b   = opt_list(&map_base);
            let map_p   = opt_list(&map_proj);
            let map_d   = opt_list(&map_ds);
            commands::import::execute(
                &file_path,
                &mode_s,
                &map_b,
                &map_p,
                &map_d,
                &database,
                ids
            )?
        }
        Some(cli::Command::Migrate { dry_run, .. }) => {
            commands::migrate::execute(dry_run, &database, ids)?
        }
        Some(cli::Command::Use { context_str }) => {
            commands::r#use::execute(&context_str)?
        }
        Some(cli::Command::Install {}) => {
            // already handled above
        }
        None => {
            // no-op
        }
    }

    Ok(())
}
