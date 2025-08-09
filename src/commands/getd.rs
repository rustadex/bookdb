use crate::error::{Result, BookdbError};
use crate::context::ResolvedContextIds;
use crate::db::Database;
use std::io::Write;

pub fn execute(dik: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Document { ds_id, .. } = ids {
        let (doc_key, seg_path) = dik.split_once('.').unwrap_or((dik, "_root"));
        match db.get_doc_segment(doc_key, seg_path, ds_id)? {
            Some((bytes, mime)) => {
                if mime.starts_with("text/") {
                    let s = String::from_utf8(bytes).unwrap_or_default();
                    println!("{}", s);
                } else {
                    std::io::stdout().write_all(&bytes)?;
                }
                Ok(())
            }
            None => Err(BookdbError::KeyNotFound(format!("{}.{}", doc_key, seg_path))),
        }
    } else { Err(BookdbError::ContextParse("`getd` requires a document context".into())) }
}
