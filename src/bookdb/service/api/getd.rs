use crate::error::{Result, BookdbError};
use crate::bookdb::service::ctx::ResolvedContext;
use crate::bookdb::service::db::Database;
use std::io::Write;

pub fn execute(dik: &str, context: &ResolvedContext, db: &Database) ->  Result<(), E> {
    let (doc_key, seg_path) = dik.split_once('.').unwrap_or((dik, "_root"));
    
    match db.get_doc_segment(doc_key, seg_path, context)? {
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
}
