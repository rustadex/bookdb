// src/commands/export.rs
use crate::db::Database;
use crate::error::Result;
use crate::context::ResolvedContextIds;
use std::fs::File;
use std::io::{Write, BufWriter, Read};

pub fn execute(
    file_path: &std::path::Path,
    format: Option<String>,
    filters: (Option<String>,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>),
    db: &Database,
    ids: ResolvedContextIds,
) -> Result<()> {
    let fmt = format.unwrap_or_else(|| "kv".to_string());
    match fmt.as_str() {
        "kv" => export_kv(file_path, db, ids),
        "jsonl" => export_jsonl(file_path, filters, db, ids),
        other => { eprintln!("unknown format: {}", other); export_kv(file_path, db, ids) }
    }
}

fn export_kv(path: &std::path::Path, db: &Database, ids: ResolvedContextIds) -> Result<()> {
    let vs_id = match ids { ResolvedContextIds::Variables{ vs_id, .. } => vs_id, _ => {
        // if not in var context, no vars to export
        let mut w = BufWriter::new(File::create(path)?);
        w.flush()?;
        return Ok(());
    }};
    let rows = db.stream_vars(vs_id)?;
    let mut w = BufWriter::new(File::create(path)?);
    for (k,v) in rows {
        writeln!(&mut w, "{}={}", k, v)?;
    }
    w.flush()?;
    Ok(())
}

fn export_jsonl(
    path: &std::path::Path,
    _filters: (Option<String>,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>),
    db: &Database,
    ids: ResolvedContextIds,
) -> Result<()> {
    use serde::Serialize;
    #[derive(Serialize)]
    #[serde(tag="type")]
    enum Row<'a>{
        Var{ key:&'a str, value:&'a str },
        Doc{ doc:&'a str },
        Segment{ doc:&'a str, path:&'a str, mime:&'a str, content_b64:String },
    }
    let mut w = BufWriter::new(File::create(path)?);
    if let ResolvedContextIds::Variables{ vs_id, .. } = ids {
        let rows = db.stream_vars(vs_id)?;
        for (k,v) in rows.iter() {
            serde_json::to_writer(&mut w, &Row::Var{ key:k, value:v })?; writeln!(&mut w)?;
        }
    }
    let ds_id = match ids { ResolvedContextIds::Variables { ds_id, .. } | ResolvedContextIds::Document { ds_id, .. } => ds_id };
    for d in db.list_docs_v2(ds_id)? {
        let segs = db.list_segments(&d, ds_id).unwrap_or_default();
        if segs.is_empty() {
            serde_json::to_writer(&mut w, &Row::Doc{ doc:&d })?; writeln!(&mut w)?;
        } else {
            for path_s in segs {
                if let Some((bytes,mime)) = db.get_doc_segment(&d, &path_s, ds_id)? {
                    let b64 = base64::encode(bytes);
                    serde_json::to_writer(&mut w, &Row::Segment{ doc:&d, path:&path_s, mime:&mime, content_b64:b64 })?; writeln!(&mut w)?;
                }
            }
        }
    }
    w.flush()?;
    Ok(())
}
