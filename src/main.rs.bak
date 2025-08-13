// src/main.rs — BookDB with ODX initialization
#![allow(dead_code)]
#![allow(unused_imports)]

use clap::Parser;
use std::process;
use stderr::{Stderr, StderrConfig}; 

mod bookdb; // this points at src/bookdb/mod.rs

// Pull CLI from where it actually lives
use bookdb::app::ctrl::cli::{Cli, Commands};
// ODX init & config
use bookdb::oxidize::{init_from_cli, OxidexConfig};

fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // === ODX Framework Initialization ===
    let config = init_from_cli(&cli);

    // Initialize logger (uses ODX env)
    let mut logger = stderr::Stderr::new();

    // Show startup info in trace mode
    if config.show_trace() {
        logger.trace(&format!("BookDB v{} starting", env!("CARGO_PKG_VERSION")));
        logger.trace(&format!("ODX config: {:?}", config));

        if config.is_dev_mode() {
            bookdb::oxidize::print_environment_status();
        }
    }

    // Run the requested command
    if let Err(e) = run_command(&cli, &config, &mut logger) {
        logger.error(&format!("Error: {}", e));
        process::exit(1);
    }
}

fn run_command(
    cli: &Cli,
    config: &OxidexConfig,
    logger: &mut stderr::Stderr,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get database path (CLI override or default)
    let db_path = config.get_db_path("bookdb.sqlite");

    // Get base context (CLI override or default)
    let base_context = config.get_base_context("default");

    if config.show_trace() {
        logger.trace(&format!("Using database: {}", db_path));
        logger.trace(&format!("Base context: {}", base_context));
    }

    // TODO: dispatch by `cli.command` here

    Ok(()) // <- satisfy Result<(), _>
}
