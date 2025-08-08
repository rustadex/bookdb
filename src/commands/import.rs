// src/commands/import.rs
use crate::db::Database;
use crate::error::{BookdbError, Result};
use crate::context::ResolvedContextIds;
use std::fs::File;
use std::io::{BufRead,BufReader};

pub fn execute(
    file_path: &std::path::Path,
    mode: &str,
    _map_base: &[String],
    _map_proj: &[String],
    _map_ds: &[String],
    format: &Option<String>,
    db: &Database,
    ids: ResolvedContextIds,
) -> Result<()> {
    let fmt = format.clone().unwrap_or_else(|| detect_format(file_path)?);
    match fmt.as_str() {
        "kv" => import_kv(file_path, mode, db, ids),
        "jsonl" => import_jsonl(file_path, mode, db, ids),
        _ => Err(BookdbError::Argument("unknown import format".into())),
    }
}

fn detect_format(p: &std::path::Path) -> Result<String> {
    let f = File::open(p)?; let mut r = BufReader::new(f);
    let mut buf = String::new();
    while r.read_line(&mut buf)?>0 {
        let s = buf.trim_start();
        if s.is_empty() { buf.clear(); continue; }
        if s.starts_with('{') { return Ok("jsonl".into()); } else { return Ok("kv".into()); }
    }
    Ok("kv".into())
}

fn import_kv(p: &std::path::Path, _mode: &str, db: &Database, ids: ResolvedContextIds)->Result<()> {
    let vs_id = match ids { ResolvedContextIds::Variables{ vs_id, .. } => vs_id, _ => return Ok(()) };
    let f = File::open(p)?; let r = BufReader::new(f);
    for line in r.lines() {
        let line = line?; if line.trim().is_empty() || line.trim_start().starts_with('#') { continue; }
        if let Some((k,v)) = line.split_once('=') {
            db.set_var(k.trim(), v.trim(), vs_id)?;
        }
    }
    Ok(())
}

fn import_jsonl(p: &std::path::Path, mode: &str, db: &Database, ids: ResolvedContextIds)->Result<()> {
    #[derive(serde::Deserialize)] #[serde(tag="type")]
    enum Row { Var{ key:String, value:String }, Doc{ doc:String }, Segment{ doc:String, path:String, mime:String, content_b64:String } }
    let ds_id = match ids { ResolvedContextIds::Variables{ ds_id, .. } | ResolvedContextIds::Document{ ds_id, .. } => ds_id };
    let vs_id_opt = match ids { ResolvedContextIds::Variables{ vs_id, .. } => Some(vs_id), _ => None };
    let overwrite = mode=="overwrite";
    let f = File::open(p)?; let r = BufReader::new(f);
    for line in r.lines() {
        let l = line?; if l.trim().is_empty() { continue; }
        let row: Row = serde_json::from_str(&l).map_err(|e| BookdbError::Argument(format!("bad json: {e}")))?;
        match row {
            Row::Var{ key, value } => {
                if let Some(vs_id)=vs_id_opt { db.set_var(&key, &value, vs_id)?; }
            }
            Row::Doc{ doc } => { db.set_doc_segment(&doc, "_root", "text/plain", b"", ds_id)?; }
            Row::Segment{ doc, path, mime, content_b64 } => {
                let bytes = base64::decode(content_b64).map_err(|e| BookdbError::Argument(format!("b64: {e}")))?;
                db.set_doc_segment(&doc, &path, &mime, &bytes, ds_id)?;
            }
        }
    }
    Ok(())
}
