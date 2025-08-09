use crate::error::Result;
use crate::context::ResolvedContextIds;
use crate::db::Database;

pub fn execute(dik_value: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    let (dik, value) = dik_value.split_once('=').ok_or_else(|| crate::error::BookdbError::Argument("expected dik=value".into()))?;
    let (doc_key, seg_path) = dik.split_once('.').unwrap_or((dik, "_root"));
    if let ResolvedContextIds::Document { ds_id, .. } = ids {
        db.set_doc_segment(doc_key, seg_path, "text/plain", value.as_bytes(), ds_id)?;
        println!("Ok.");
        Ok(())
    } else { Err(crate::error::BookdbError::ContextParse("`setd` requires a document context".into())) }
}
