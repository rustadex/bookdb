// src/cli.rs

use clap::{Parser, Subcommand, ValueEnum}
    Migrate { #[arg(long)] dry_run: bool, #[arg(long)] backup: Option<PathBuf> },
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "A stateful, context-aware key-value store.")]
pub struct Cli {
    #[arg(long, global=true)] pub debug: bool,
    #[arg(long, global=true)] pub trace: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Getv { key: String },
    Setv {
        #[arg(help = "The variable to set, in KEY=VALUE format")]
        key_value: String,
    },
    Getd {
        #[arg(help = "The Document Info Key (e.g., 'summary')")]
        dik: String,
    },
    Setd {
        #[arg(help = "The document chunk to set, in DIK=VALUE format")]
        dik_value: String,
    },
    Ls {
        #[command(subcommand)]
        target: LsTarget,
    },
    Cursor,
    /// Set and persist a new context.
    Use {
        #[arg(help = "The context to set (e.g., 'myapp.api.var.secrets')")]
        context_str: String,
    },
    Import {
    file_path: PathBuf,
    #[arg(long, default_value="merge")] mode: String,
    #[arg(long)] map_base: Vec<String>,
    #[arg(long)] map_proj: Vec<String>,
    #[arg(long)] map_ds: Vec<String>,
    #[arg(long)] format: Option<String>,
},
    Export {
    file_path: PathBuf,
    #[arg(long)] format: Option<String>,
    #[arg(long)] proj: Option<String>,
    #[arg(long)] ds: Option<String>,
    #[arg(long)] vs: Option<String>,
    #[arg(long)] doc: Option<String>,
    #[arg(long)] key: Option<String>,
    #[arg(long)] seg: Option<String>,
},
}

#[derive(Subcommand, Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum LsTarget {
    Keys,
    Docs,
    Varstores,
    Docstores,
    Projects,
}