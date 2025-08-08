// src/commands/ls.rs

use crate::cli::LsTarget;
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::error::{BookdbError, Result};

pub fn execute_projects(db: &Database) -> Result<()> {
    let items = db.list_projects()?;
    print_items(items);
    Ok(())
}

pub fn execute(target: LsTarget, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    let items = match target {
        LsTarget::Projects => unreachable!(),
        LsTarget::Docstores => {
            let p_id = match ids {
                ResolvedContextIds::Variables { p_id, .. } | ResolvedContextIds::Document { p_id, .. } => p_id,
            };
            db.list_docstores(p_id)?
        }
        LsTarget::Varstores => {
            let ds_id = match ids {
                ResolvedContextIds::Variables { ds_id, .. } | ResolvedContextIds::Document { ds_id, .. } => ds_id,
            };
            db.list_varstores(ds_id)?
        }
        LsTarget::Keys => {
            if let ResolvedContextIds::Variables { vs_id, .. } = ids {
                db.list_keys(vs_id)?
            } else {
                return Err(BookdbError::ContextParse("Cannot list keys in a document context.".into()));
            }
        }
        LsTarget::Docs => {
            let ds_id = match ids { ResolvedContextIds::Variables { ds_id, .. } | ResolvedContextIds::Document { ds_id, .. } => ds_id };
            return { let items = db.list_docs_v2(ds_id)?; print_items(items); Ok(()) };
            if let ResolvedContextIds::Document { ds_id, .. } = ids {
                db.list_diks(ds_id)?
            } else {
                return Err(BookdbError::ContextParse("Cannot list docs in a variable context.".into()));
            }
        }
    };

    print_items(items);
    Ok(())
}

fn print_items(items: Vec<String>) {
    if items.is_empty() {
        println!("(empty)");
    } else {
        for item in items {
            println!("{}", item);
        }
    }
}
