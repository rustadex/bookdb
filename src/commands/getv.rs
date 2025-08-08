// src/commands/getv.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};

pub fn execute(key: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Variables { vs_id, .. } = ids {
        match db.get_var(key, vs_id)? {
            Some(value) => {
                println!("{}", value);
                Ok(())
            }
            None => Err(BookdbError::KeyNotFound(key.to_string())),
        }
    } else {
        Err(BookdbError::ContextParse("`getv` command requires a variable context.".into()))
    }
}
