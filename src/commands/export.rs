use crate::error::Result;
use crate::context::ResolvedContextIds;
use crate::db::Database;
use serde::Serialize;
use std::fs::File;
use std::io::{Write, BufWriter};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize)]
#[serde(tag="type")]
enum Row<'a> {
    #[allow(dead_code)]
    Var{ project:&'a str, docstore:&'a str, varstore:&'a str, key:&'a str, value:&'a str },
    Doc{ project:&'a str, docstore:&'a str, doc:&'a str },
    Segment{ project:&'a str, docstore:&'a str, doc:&'a str, path:&'a str, mime:&'a str, content_b64:&'a str },
}

pub fn execute(
    file_path: &std::path::PathBuf,
    format: Option<String>,
    _filters: (Option<String>,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>),
    db: &Database,
    ids: ResolvedContextIds
) -> Result<()> {
    let fmt = format.unwrap_or_else(|| "kv".into());
    let mut w = BufWriter::new(File::create(file_path)?);

    match ids {
        ResolvedContextIds::Variables { _project_id: _, vs_id } => {
            if fmt == "kv" {
                let rows = db.list_keys(vs_id)?;
                for k in rows {
                    if let Some(v) = db.get_var(&k, vs_id)? {
                        writeln!(w, "{}={}", k, v)?;
                    }
                }
            } else { /* jsonl Var rows could be added later */ }
        }
        ResolvedContextIds::Document { _project_id: _, ds_id } => {
            let docs = db.list_docs_v2(ds_id)?;
            for d in docs.iter() {
                if fmt == "jsonl" {
                    let row = Row::Doc{ project:"", docstore:"", doc: d };
                    serde_json::to_writer(&mut w, &row)?; w.write_all(b"\n")?;
                }
                let segs = db.list_segments(d, ds_id)?;
                for path in segs.iter() {
                    if let Some((bytes, mime)) = db.get_doc_segment(d, path, ds_id)? {
                        let b64 = general_purpose::STANDARD.encode(bytes);
                        if fmt == "jsonl" {
                            let row = Row::Segment{ project:"", docstore:"", doc: d, path, mime: &mime, content_b64: &b64 };
                            serde_json::to_writer(&mut w, &row)?; w.write_all(b"\n")?;
                        }
                    }
                }
            }
        }
    }
    w.flush()?;
    Ok(())
}
