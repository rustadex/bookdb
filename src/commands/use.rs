// src/commands/use.rs

use crate::context;
use crate::error::Result;
use log::info;
use std::path::PathBuf;

pub fn execute(context_str: &str, cursor_path: &PathBuf) -> Result<()> {
    let new_context = context::parse_context_string(context_str)?;
    context::save_context(&new_context, cursor_path)?;
    info!("Persisted new context: {:?}", new_context);
    println!("Context set to: {}", context_str);
    Ok(())
}
