use crate::error::Result;
use crate::context::ResolvedContext;
use crate::db::Database;

pub fn execute(dik_value: &str, context: &ResolvedContext, db: &Database) ->  Result<(), E> {
    let (dik, value) = dik_value.split_once('=')
        .ok_or_else(|| crate::error::BookdbError::Argument("Expected dik=value format".into()))?;
    
    let (doc_key, seg_path) = dik.split_once('.').unwrap_or((dik, "_root"));
    
    db.set_doc_segment(doc_key, seg_path, "text/plain", value.as_bytes(), context)?;
    println!("Ok.");
    Ok(())
}
