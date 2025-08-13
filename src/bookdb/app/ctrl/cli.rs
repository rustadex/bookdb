// src/cli.rs - Root CLI module (ODX-based from Session 18)

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "bookdb")]
#[command(about = "A CLI tool for managing key-value stores and documents")]
#[command(version)]
pub struct Cli {
    /// Enable debug mode (shows info, warn, okay messages)
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Enable trace mode (shows trace, think messages + debug)
    #[arg(short = 't', long)]
    pub trace: bool,

    /// Quiet mode (only error/fatal messages)
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Force mode (bypass safety guards)
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Auto-confirm prompts
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Developer mode (enables debug + trace + dev features)
    #[arg(short = 'D', long)]
    pub dev: bool,

    /// Dry run mode (show what would be done)
    #[arg(long)]
    pub dry_run: bool,

    /// JSON output for machine consumption
    #[arg(long)]
    pub json: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Override database path
    #[arg(long)]
    pub db_path: Option<String>,

    /// Override base context
    #[arg(long)]
    pub base: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get a variable value
    Getv {
        /// Variable key
        key: String,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Set a variable value
    Setv {
        /// Key=value pair
        key_value: String,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Delete a variable
    Delv {
        /// Variable key
        key: String,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Increment a numeric variable
    Inc {
        /// Variable key
        key: String,
        /// Amount to increment (default: 1)
        amount: Option<i64>,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Decrement a numeric variable
    Dec {
        /// Variable key
        key: String,
        /// Amount to decrement (default: 1)
        amount: Option<i64>,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// List data (projects, keys, etc.)
    Ls {
        /// What to list
        target: LsTarget,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Get a document
    Getd {
        /// Document key
        dik: String,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Set a document
    Setd {
        /// Document key=value pair
        dik_value: String,
        /// Context chain override
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Export data to file
    Export {
        /// Output file path
        file_path: String,
        /// Export format
        #[arg(short, long)]
        format: Option<String>,
        /// Project filter
        #[arg(long)]
        proj: Option<String>,
        /// Workspace filter
        #[arg(long)]
        workspace: Option<String>,
        /// Keystore filter
        #[arg(long)]
        keystore: Option<String>,
        /// Document filter
        #[arg(long)]
        doc: Option<String>,
        /// Key filter
        #[arg(long)]
        key: Option<String>,
        /// Segment filter
        #[arg(long)]
        seg: Option<String>,
    },
    /// Import data from file
    Import {
        /// Input file path
        file_path: String,
        /// Import mode
        #[arg(short, long)]
        mode: Option<String>,
        /// Map base
        #[arg(long)]
        map_base: Option<String>,
        /// Map project
        #[arg(long)]
        map_proj: Option<String>,
        /// Map workspace
        #[arg(long)]
        map_workspace: Option<String>,
    },
    /// Migrate data
    Migrate {
        /// Show what would be migrated without doing it
        #[arg(long)]
        dry_run: bool,
    },
    /// Change active context
    Use {
        /// New context string
        context_str: String,
    },
    /// Install BookDB
    Install {},
    /// Show current cursor/context
    Cursor {},
    /// Show current status
    Status {},
    /// Show current base
    Base {},
    /// Find a key across projects
    Find {
        /// Search pattern
        pattern: String,
    },
}

#[derive(ValueEnum, Clone)]
pub enum LsTarget {
    /// List all available data
    All,
    /// List projects
    Projects,
    /// List workspaces
    Workspaces,
    /// List keystores
    Keystores,
    /// List variable keys
    Keys,
    /// List documents
    Docs,
    /// List bases
    Bases,
}

impl Default for LsTarget {
    fn default() -> Self {
        Self::All
    }
}
