use crate::error::Result;
use crate::bookdb::service::ctx::ResolvedContext;
use crate::bookdb::service::db::Database;

pub fn execute(dry_run: bool, context: &ResolvedContext, db: &Database) ->  Result<(), E> {
    let chunks = db.stream_doc_chunks(context)?;
    
    if chunks.is_empty() {
        println!("No legacy doc_chunks found");
        return Ok(());
    }
    
    for (doc_key, value) in chunks {
        println!("Migrating {} -> docs/_root", doc_key);
        if !dry_run {
            db.set_doc_segment(&doc_key, "_root", "text/plain", value.as_bytes(), context)?;
        }
    }
    
    if dry_run {
        println!("Dry-run: no changes written");
    } else {
        println!("Migration completed");
    }
    
    Ok(())
}
