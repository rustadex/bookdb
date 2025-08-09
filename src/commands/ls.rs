use crate::error::Result;
use crate::context::ResolvedContextIds;
use crate::db::Database;
use crate::cli::LsTarget;

fn print_items(items: Vec<String>) {
    if items.is_empty() { println!("(empty)"); }
    else { for s in items { println!("{}", s); } }
}

pub fn execute(target: LsTarget, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    match target {
        LsTarget::Projects  => { let v = db.list_projects()?; print_items(v); Ok(()) }
        LsTarget::Docstores => { let v = db.list_docstores()?; print_items(v); Ok(()) }
        LsTarget::Varstores => { let v = db.list_varstores()?; print_items(v); Ok(()) }
        LsTarget::Docs => {
            if let ResolvedContextIds::Document { ds_id, .. } = ids {
                let v = db.list_docs_v2(ds_id)?; print_items(v); Ok(())
            } else { println!("(empty)"); Ok(()) }
        }
        LsTarget::Keys => {
            if let ResolvedContextIds::Variables { vs_id, .. } = ids {
                let v = db.list_keys(vs_id)?; print_items(v); Ok(())
            } else { println!("(empty)"); Ok(()) }
        }
    }
}
