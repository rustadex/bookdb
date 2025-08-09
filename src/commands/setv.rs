use crate::error::Result;
use crate::context::ResolvedContextIds;
use crate::db::Database;

pub fn execute(key_value: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    let (k,v) = key_value.split_once('=').ok_or_else(|| crate::error::BookdbError::Argument("expected key=value".into()))?;
    if let ResolvedContextIds::Variables { vs_id, .. } = ids {
        db.set_var(k, v, vs_id)?;
        println!("Ok.");
        Ok(())
    } else { Err(crate::error::BookdbError::ContextParse("`setv` requires a variable context".into())) }
}
