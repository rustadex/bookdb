use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};

pub fn execute(key_value: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Variables { vs_id, .. } = ids {
        let (k, v) = key_value.split_once('=').ok_or_else(|| BookdbError::Argument("Use KEY=VALUE".into()))?;
        db.set_var(k, v, vs_id)?;
        println!("Ok.");
        Ok(())
    } else {
        Err(BookdbError::ContextParse("`setv` requires a variable context".into()))
    }
}
