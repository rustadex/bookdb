// src/cursor.rs
use std::fs;
use crate::config::Paths;
use crate::error::Result;

pub fn read_cursor(paths: &Paths) -> (Option<String>, Option<String>) {
    let base = fs::read_to_string(&paths.cursor_base_path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let chain = fs::read_to_string(&paths.cursor_chain_path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    (base, chain)
}

pub fn write_cursor(paths: &Paths, base: Option<&str>, chain: Option<&str>) -> Result<()> {
    if let Some(b) = base { if !b.is_empty() { fs::write(&paths.cursor_base_path, b)?; } }
    if let Some(c) = chain { if !c.is_empty() { fs::write(&paths.cursor_chain_path, c)?; } }
    Ok(())
}
