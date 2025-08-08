// src/commands/import.rs
use crate::db::Database;
use crate::error::{BookdbError, Result};
use crate::context::ResolvedContextIds;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use base64::{engine::general_purpose, Engine as _};

#[derive(Deserialize)]
#[serde(tag="type")]
enum Row {
    Var { _varstore: String, key: String, value: String, _project: Option<String>, _docstore: Option<String> },
    Doc { doc: String, _project: Option<String>, _docstore: Option<String> },
    Segment { doc: String, path: String, mime: String, content_b64: String, _project: Option<String>, _docstore: Option<String> },
}

fn detect_format(path: &std::path::Path) -> String {
    if let Ok(mut f) = File::open(path) {
        let mut buf = [0u8; 1];
        while let Ok(n) = f.read(&mut buf) {
            if n == 0 { break; }
            let c = buf[0] as char;
            if c.is_whitespace() { continue; }
            return if c == '{' { "jsonl".into() } else { "kv".into() };
        }
    }
    "kv".into()
}

pub fn execute(
    file_path: &std::path::Path,
    mode: &str,
    _map_base: &[String],
    _map_proj: &[String],
    _map_ds: &[String],
    db: &Database,
    ids: ResolvedContextIds,
) -> Result<()> {
    let fmt = detect_format(file_path);
    let ds_id = match ids { ResolvedContextIds::Variables { ds_id, .. } | ResolvedContextIds::Document { ds_id, .. } => ds_id };
    let _overwrite = matches!(mode, "overwrite");

    if fmt == "kv" {
        let vs_id = match ids { ResolvedContextIds::Variables{ vs_id, .. } => vs_id, _ => return Err(BookdbError::Argument("KV import requires VAR context".into())) };
        let rdr = BufReader::new(File::open(file_path)?);
        for line in rdr.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') { continue; }
            if let Some((k,v)) = line.split_once('=') {
                let (k, v) = (k.trim(), v.trim());
                if !k.is_empty() { db.set_var(k, v, vs_id)?; }
            }
        }
        return Ok(());
    }

    // JSONL
    let f = File::open(file_path)?;
    let mut rdr = BufReader::new(f);
    let mut line = String::new();
    while rdr.read_line(&mut line)? > 0 {
        let ln = line.trim();
        if ln.is_empty() { line.clear(); continue; }
        let row: Row = serde_json::from_str(ln).map_err(|e| BookdbError::Argument(format!("bad json: {e}")))?;
        match row {
            Row::Var{ key, value, .. } => {
                if let ResolvedContextIds::Variables{ vs_id, .. } = ids { db.set_var(&key, &value, vs_id)?; }
            }
            Row::Doc{ doc, .. } => { db.set_doc_segment(&doc, "_root", "text/plain", b"", ds_id)?; }
            Row::Segment{ doc, path, mime, content_b64, .. } => {
                let bytes = general_purpose::STANDARD.decode(content_b64).map_err(|e| BookdbError::Argument(format!("b64: {e}")))?;
                db.set_doc_segment(&doc, &path, &mime, &bytes, ds_id)?;
            }
        }
        line.clear();
    }
    Ok(())
}