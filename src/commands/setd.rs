use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};

pub fn execute(dik_value: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Document { ds_id } = ids {
        let (dik, value) = dik_value.split_once('=').ok_or_else(|| BookdbError::Argument("Use DOC_KEY[.segment]=VALUE".into()))?;
        let mut parts = dik.splitn(2, '.');
        let doc_key = parts.next().unwrap_or("");
        let seg_path = parts.next().unwrap_or("_root");
        db.set_doc_segment(doc_key, seg_path, "text/plain", value.as_bytes(), ds_id)?;
        println!("Ok.");
        Ok(())
    } else { Err(BookdbError::ContextParse("`setd` requires a document context".into())) }
}
