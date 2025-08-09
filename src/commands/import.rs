use crate::error::{Result, BookdbError};
use crate::context::ResolvedContextIds;
use crate::db::Database;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn execute(
    file_path: &std::path::PathBuf,
    mode: &String,
    _map_base: &Vec<String>,
    _map_proj: &Vec<String>,
    _map_ds: &Vec<String>,
    db: &Database,
    ids: ResolvedContextIds
) -> Result<()> {
    if mode != "merge" { return Err(BookdbError::Argument("only --mode=merge supported in refimpl".into())); }

    if let ResolvedContextIds::Variables { vs_id, .. } = ids {
        let rdr = BufReader::new(File::open(file_path)?);
        for line in rdr.lines() {
            let line = line?;
            if let Some((k,v)) = line.split_once('=') {
                if !k.trim().is_empty() { db.set_var(k.trim(), v.trim(), vs_id)?; }
            }
        }
        println!("Ok.");
        Ok(())
    } else if let ResolvedContextIds::Document { .. } = ids {
        return Err(BookdbError::Argument("document import not implemented in refimpl".into()));
    } else {
        Err(BookdbError::ContextParse("import requires a context".into()))
    }
}
