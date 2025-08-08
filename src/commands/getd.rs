use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};
use std::io::Write;

pub fn execute(dik: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Document { ds_id } = ids {
        let mut parts = dik.splitn(2, '.');
        let doc_key = parts.next().unwrap_or("");
        let seg_path = parts.next().unwrap_or("_root");
        match db.get_doc_segment(doc_key, seg_path, ds_id)? {
            Some((bytes, _mime)) => {
                match String::from_utf8(bytes.clone()) { Ok(s) => println!("{}", s), Err(_) => std::io::stdout().write_all(&bytes)? };
                Ok(())
            }
            None => Err(BookdbError::KeyNotFound(format!("{}.{}", doc_key, seg_path))),
        }
    } else { Err(BookdbError::ContextParse("`getd` requires a document context".into())) }
}
