// src/commands/getd.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};

pub fn execute(dik: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Document { ds_id, .. } = ids {
        match db.get_doc_chunk(dik, ds_id)? {
            Some(value) => {
                println!("{}", value);
                Ok(())
            }
            None => Err(BookdbError::KeyNotFound(dik.to_string())),
        }
    } else {
        Err(BookdbError::ContextParse("`getd` command requires a document context.".into()))
    }
}
