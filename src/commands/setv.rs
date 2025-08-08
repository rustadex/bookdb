// src/commands/setv.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};
use log::info;

pub fn execute(key_value: &str, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    if let ResolvedContextIds::Variables { vs_id, .. } = ids {
        let (key, value) = key_value.split_once('=').ok_or_else(|| {
            BookdbError::Argument("Invalid format. Use KEY=VALUE.".to_string())
        })?;

        db.set_var(key, value, vs_id)?;

        info!("Successfully set '{}'", key);
        println!("Ok.");
        Ok(())
    } else {
        Err(BookdbError::ContextParse("`setv` command requires a variable context.".into()))
    }
}
