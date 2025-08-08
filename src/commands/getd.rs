// src/commands/getd.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};

pub fn execute(dik: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Document { ds_id, .. } = ids {
        let mut parts = dik.splitn(2, '.'); let doc_key=parts.next().unwrap_or(""); let seg=parts.next().unwrap_or("_root"); match db.get_doc_segment(doc_key, seg, ds_id)? {
            Some((bytes, _mime)) => {
                match String::from_utf8(bytes.clone()) { Ok(s)=>println!("{}", s), Err(_)=>{ use std::io::Write; std::io::stdout().write_all(&bytes).unwrap(); } }
                Ok(())
            }
            None => Err(BookdbError::KeyNotFound(dik.to_string())),
        }
    } else {
        Err(BookdbError::ContextParse("`getd` command requires a document context.".into()))
    }
}
