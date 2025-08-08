// src/commands/ls.rs
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};
use crate::cli::LsTarget;

fn print_items(items: Vec<String>) {
    for it in items { println!("{}", it); }
}

pub fn execute(target: LsTarget, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    match target {
        LsTarget::Keys => {
            if let ResolvedContextIds::Variables { vs_id, .. } = ids {
                let items = db.list_keys(vs_id)?; print_items(items); Ok(())
            } else { Err(BookdbError::ContextParse("Cannot list keys in a document context".into())) }
        }
        LsTarget::Docs => {
            let ds_id = match ids { ResolvedContextIds::Variables { ds_id, .. } | ResolvedContextIds::Document { ds_id, .. } => ds_id };
            let items = db.list_docs_v2(ds_id)?; print_items(items); Ok(())
        }
        LsTarget::Varstores => { let items = db.list_varstores()?; print_items(items); Ok(()) }
        LsTarget::Docstores => { let items = db.list_docstores()?; print_items(items); Ok(()) }
        LsTarget::Projects => { let items = db.list_projects()?; print_items(items); Ok(()) }
    }
}
