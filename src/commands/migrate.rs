use crate::{span, log_info, log_debug, log_warn};
use crate::rdx_stderr::Level;
use crate::error::Result;
use crate::context::ResolvedContextIds;
use crate::db::Database;

pub fn execute(dry_run: bool, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    span!(Level::Info, "migrate v1->v2 (noop in refimpl)");
    if let ResolvedContextIds::Document { ds_id, .. } = ids {
        let chunks = db.stream_doc_chunks(ds_id)?;
        if chunks.is_empty() { log_info!("no legacy doc_chunks found"); }
        for (doc_key, value) in chunks {
            log_debug!("migrate {} -> docs/_root", doc_key);
            if !dry_run {
                db.set_doc_segment(&doc_key, "_root", "text/plain", value.as_bytes(), ds_id)?;
            }
        }
        if dry_run { log_warn!("dry-run: no changes written"); } else { log_info!("migration completed"); }
    }
    Ok(())
}
