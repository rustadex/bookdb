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

        db.set_doc_chunk(dik, value, ds_id)?;

        info!("Successfully set DIK '{}'", dik);
        println!("Ok.");
        Ok(())
    } else {
        Err(BookdbError::ContextParse("`setd` command requires a document context.".into()))
    }
}
