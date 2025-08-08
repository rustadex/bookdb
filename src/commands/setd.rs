// src/commands/setd.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};
use log::info;

pub fn execute(dik_value: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Document { ds_id, .. } = ids {
        let (dik, value) = dik_value.split_once('=').ok_or_else(|| {
            BookdbError::Argument("Invalid format. Use DIK=VALUE.".to_string())
        })?;

        let mut parts = dik.splitn(2, '.'); let doc_key=parts.next().unwrap_or(""); let seg=parts.next().unwrap_or("_root"); db.set_doc_segment(doc_key, seg, "text/plain", value.as_bytes(), ds_id)?;

        info!("Successfully set DIK '{}'", dik);
        println!("Ok.");
        Ok(())
    } else {
        Err(BookdbError::ContextParse("`setd` command requires a document context.".into()))
    }
}
