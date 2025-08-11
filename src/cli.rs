// src/cli.rs - Updated with consistent BOOKDB_CONCEPTS.md terminology

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(name="bookdb")]
#[command(about="bookdb")]
pub struct Cli {
    /// Enable debug logging to stderr
    #[arg(long)]
    pub debug: bool,

    /// Enable trace logging to stderr
    #[arg(long)]
    pub trace: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Get variable: KEY [CONTEXT]
    Getv {
        /// variable key (final tail segment if not embedded)
        key: String,
        /// context chain as the LAST arg: BASE@PROJECT.WORKSPACE.VAR.KEYSTORE[.KEY]
        context_chain: Option<String>,
    },
    /// Set variable: KEY=VALUE [CONTEXT]
    Setv {
        /// key=value (VALUE may be quoted)
        key_value: String,
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
        map_workspace: Option<String>,               // FIXED: was map_ds
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
        workspace: Option<String>,                   // FIXED: was ds
        #[arg(long)]
        keystore: Option<String>,                    // FIXED: was vs
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
    Workspaces,     // FIXED: was Docstores - List workspaces in current project
    Keystores       // FIXED: was Varstores - List keystores in current workspace
}
