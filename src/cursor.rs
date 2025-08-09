use crate::config::Paths;
use crate::error::Result;
use std::fs;

pub fn read_cursor(paths: &Paths) -> (Option<String>, Option<String>) {
    let base = fs::read_to_string(&paths.cursor_base_path).ok()
        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let chain = fs::read_to_string(&paths.cursor_chain_path).ok()
        .map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    (base, chain)
}

pub fn write_cursor(paths: &Paths, base_db_abs: Option<&str>, chain_full: Option<&str>) -> Result<()> {
    if let Some(b) = base_db_abs { if !b.is_empty() { fs::write(&paths.cursor_base_path, b)?; } }
    if let Some(c) = chain_full { if !c.is_empty() { fs::write(&paths.cursor_chain_path, c)?; } }
    Ok(())
}
