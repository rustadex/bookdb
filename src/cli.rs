// src/cli.rs
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
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

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Getv { key: String },
    Setv { key_value: String },
    Getd { dik: String },                  // doc_key[.segment]
    Setd { dik_value: String },            // doc_key[.segment]=VALUE
    Ls { target: LsTarget },
    Import {
        file_path: PathBuf,
        #[arg(long, default_value = "merge")]
        mode: String,
        #[arg(long)] map_base: Vec<String>,
        #[arg(long)] map_proj: Vec<String>,
        #[arg(long)] map_ds: Vec<String>,
        #[arg(long)] format: Option<String>, // kv|jsonl (auto if omitted)
    },
    Export {
        file_path: PathBuf,
        #[arg(long)] format: Option<String>, // kv (default) | jsonl
        #[arg(long)] proj: Option<String>,
        #[arg(long)] ds: Option<String>,
        #[arg(long)] vs: Option<String>,
        #[arg(long)] doc: Option<String>,
        #[arg(long)] key: Option<String>,
        #[arg(long)] seg: Option<String>,
    },
    Migrate { #[arg(long)] dry_run: bool },
    Use { context_str: String },
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum LsTarget { Keys, Docs, Varstores, Docstores, Projects }
