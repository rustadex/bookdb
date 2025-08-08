// src/commands/export.rs

use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::Result;
use log::info;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn execute(file_path: &PathBuf, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    info!("Exporting to file: {}", file_path.display());

    let mut file = File::create(file_path)?;

    let rows = match ids {
        ResolvedContextIds::Variables { vs_id, .. } => db.stream_vars(vs_id)?,
        ResolvedContextIds::Document { ds_id, .. } => db.stream_doc_chunks(ds_id)?,
    };

    let mut count = 0;
    for (key, value) in rows {
        writeln!(file, "{}=\"{}\"", key, value)?;
        count += 1;
    }

    println!("Export complete. {} keys written to {}.", count, file_path.display());
    Ok(())
}
