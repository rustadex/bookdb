// src/commands/import.rs

use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::Result;
use log::{info, warn};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn execute(file_path: &PathBuf, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    info!("Importing from file: {}", file_path.display());

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"');
            match ids {
                ResolvedContextIds::Variables { vs_id, .. } => {
                    db.set_var(key, value, vs_id)?;
                }
                ResolvedContextIds::Document { ds_id, .. } => {
                    db.set_doc_chunk(key, value, ds_id)?;
                }
            }
            count += 1;
        } else {
            warn!("Skipping malformed line: {}", line);
        }
    }

    println!("Import complete. {} keys processed.", count);
    Ok(())
}
