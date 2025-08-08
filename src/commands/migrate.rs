// src/commands/migrate.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::Result;
use crate::{log_info, log_debug, span};
use crate::rdx_stderr::Level;

pub fn execute(dry_run: bool, _backup: Option<std::path::PathBuf>, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    span!(Level::Info, "migrate v1->v2");
    let ds_id = match ids { ResolvedContextIds::Variables{ ds_id, .. } | ResolvedContextIds::Document{ ds_id, .. } => ds_id };
    let chunks = db.stream_doc_chunks(ds_id)?;
    if chunks.is_empty() { log_info!("no legacy doc_chunks to migrate"); return Ok(()); }
    for (doc_key, value) in chunks {
        log_debug!("migrate {} -> _root", doc_key);
        if !dry_run {
            db.set_doc_segment(&doc_key, "_root", "text/plain", value.as_bytes(), ds_id)?;
        }
    }
    Ok(())
}
