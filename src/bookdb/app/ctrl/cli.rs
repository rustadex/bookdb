use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "bookdb")]
#[command(about = "BookDB - Context-aware key-value and document store")]
pub struct Cli {
    /// Enable global context chain (parsed before command)
    #[arg(long, global = true)]
    pub context: Option<String>,
    
    /// Persist context chain as cursor for subsequent commands
    #[arg(long, global = true)]
    pub persist: bool,
    
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get variable: KEY [CONTEXT]
    Getv {
        /// variable key to retrieve
        key: String,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Set variable: KEY=VALUE [CONTEXT]
    Setv {
        /// e.g. "API_KEY=12345"
        key_value: String,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Delete variable: KEY [CONTEXT]
    Delv {
        /// variable key to delete
        key: String,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Increment numeric variable: KEY [CONTEXT]
    Inc {
        /// variable key to increment
        key: String,
        /// amount to increment by (default: 1)
        #[arg(short, long, default_value = "1")]
        amount: i64,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Decrement numeric variable: KEY [CONTEXT]
    Dec {
        /// variable key to decrement
        key: String,
        /// amount to decrement by (default: 1)
        #[arg(short, long, default_value = "1")]
        amount: i64,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Get document segment: SEGMENT_PATH [CONTEXT]
    Getd {
        /// e.g. "main._root" (final tail segment if not embedded)
        dik: String,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Set document segment: SEGMENT_PATH=VALUE [CONTEXT]
    Setd {
        /// e.g. "main._root=hello"
        dik_value: String,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// List items in current namespace or explicit context: [keys|docs|projects|workspaces|keystores] [CONTEXT]
    Ls {
        #[arg(value_enum)]
        target: LsTarget,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Import from file into context (kv or jsonl): FILE [CONTEXT]
    Import {
        /// input file path
        file_path: std::path::PathBuf,
        /// format override: kv|jsonl (default by content)
        #[arg(long)]
        mode: Option<String>,
        /// rename (optional) base/project/workspace on import
        #[arg(long)]
        map_base: Option<String>,
        #[arg(long)]
        map_proj: Option<String>,
        #[arg(long)]
        map_workspace: Option<String>,
        /// explicit format if needed (alias of --mode)
        #[arg(long)]
        format: Option<String>,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Export from context to file (kv or jsonl): FILE [CONTEXT]
    Export {
        /// output file path
        file_path: std::path::PathBuf,
        /// format: kv|jsonl
        #[arg(long)]
        format: Option<String>,
        /// optional filters (reserved for future)
        #[arg(long)]
        proj: Option<String>,
        #[arg(long)]
        workspace: Option<String>,
        #[arg(long)]
        keystore: Option<String>,
        #[arg(long)]
        doc: Option<String>,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        seg: Option<String>,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Migrate legacy doc_chunks -> v2 docs: [CONTEXT]
    Migrate {
        /// dry run (no writes)
        #[arg(long)]
        dry_run: bool,
        /// context chain as the LAST arg
        context_chain: Option<String>,
    },
    /// Set the cursor (base + chain) explicitly
    Use {
        /// full chain to persist in cursors; accepts explicit base form: BASE@ROOT.GLOBAL.VAR.MAIN
        context_str: String,
    },
    /// One-time install of the 'home' base and invincible chains
    Install {},
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LsTarget { 
    Keys,           // List variables in current keystore
    Docs,           // List documents in current workspace
    Projects,       // List all projects in current base
    Workspaces,     // List workspaces in current project
    Keystores       // List keystores in current workspace
}

/// Extract context string from command if present
fn get_context_from_command(command: &Option<cli::Command>) -> Option<String> {
    match command {
        Some(cli::Command::Getv { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Setv { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Delv { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Inc { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Dec { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Getd { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Setd { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Ls { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Import { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Export { context_chain, .. }) => context_chain.clone(),
        Some(cli::Command::Migrate { context_chain, .. }) => context_chain.clone(),
        _ => None,
    }
}


  //missing functions

        // Commands::Reset { context_chain, no_confirm } => {
        //     if config.dry_run {
        //         logger.info(&format!("DRY RUN: Would reset context '{}'", context_chain));
        //         return Ok(());
        //     }
            
        //     execute_reset(context_chain, *no_confirm, db_path, config, logger)
        // }
        
        // Commands::Validate { context_chain } => {
        //     execute_validate(context_chain, config, logger)
        // }
        
        // Commands::Stats { context_chain } => {
        //     let context = if let Some(cc) = context_chain {
        //         resolve_context_chain(&Some(cc.clone()), base_context, config, logger)?
        //     } else {
        //         base_context.to_string()
        //     };
        //     execute_stats(&context, db_path, config, logger)
        // }


// fn parse_assignment(assignment: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
//     let parts: Vec<&str> = assignment.splitn(2, '=').collect();
//     if parts.len() != 2 {
//         return Err(format!("Invalid assignment format: '{}'. Expected 'key=value'", assignment).into());
//     }
//     Ok((parts[0].to_string(), parts[1].to_string()))
// }
