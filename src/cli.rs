use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "bookdb")]
pub struct Cli {
    /// Enable debug logging to stderr
    #[arg(long, global=true)]
    pub debug: bool,

    /// Enable trace logging to stderr
    #[arg(long, global=true)]
    pub trace: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum LsTarget {
    Projects,
    Docstores,
    Varstores,
    Docs,
    Keys,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Getv { key: String, #[arg(trailing_var_arg=true)] context_chain: Vec<String> },
    Setv { key_value: String, #[arg(trailing_var_arg=true)] context_chain: Vec<String> },
    Getd { dik: String, #[arg(trailing_var_arg=true)] context_chain: Vec<String> },
    Setd { dik_value: String, #[arg(trailing_var_arg=true)] context_chain: Vec<String> },
    Ls   { target: LsTarget, #[arg(trailing_var_arg=true)] context_chain: Vec<String> },
    Import {
        file_path: PathBuf,
        #[arg(long, default_value = "merge")] mode: String,
        #[arg(long)] map_base: Vec<String>,
        #[arg(long)] map_proj: Vec<String>,
        #[arg(long)] map_ds: Vec<String>,
        #[arg(long)] format: Option<String>,
        #[arg(trailing_var_arg=true)] context_chain: Vec<String>,
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
        #[arg(trailing_var_arg=true)] context_chain: Vec<String>,
    },
    Migrate { #[arg(long)] dry_run: bool, #[arg(trailing_var_arg=true)] context_chain: Vec<String> },
    Use { context_str: String },
    Install { #[arg(long)] force: bool, #[arg(long)] db_path: Option<PathBuf> },
}
