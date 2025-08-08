// src/commands/export.rs
use crate::db::Database;
use crate::error::Result;
use crate::context::ResolvedContextIds;
use globset::{GlobBuilder, GlobMatcher};
use serde::Serialize;
use std::fs::File;
use std::io::{Write, BufWriter};
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize)]
#[serde(tag="type")]
enum Row<'a> {
    Var { project: &'a str, docstore: &'a str, varstore: &'a str, key: &'a str, value: &'a str },
    Doc { project: &'a str, docstore: &'a str, doc: &'a str },
    Segment { project: &'a str, docstore: &'a str, doc: &'a str, path: &'a str, mime: &'a str, content_b64: String },
}

struct Filters {
    _proj: Option<GlobMatcher>,
    _ds: Option<GlobMatcher>,
    _vs: Option<GlobMatcher>,
    doc: Option<GlobMatcher>,
    key: Option<GlobMatcher>,
    seg: Option<GlobMatcher>,
}
fn to_matcher(s: &Option<String>) -> Option<GlobMatcher> {
    s.as_ref().map(|p| GlobBuilder::new(p).backslash_escape(true).case_insensitive(true).build().unwrap().compile_matcher())
}

pub fn execute(
    file_path: &std::path::Path,
    format: Option<String>,
    filters_in: (Option<String>,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>),
    db: &Database,
    ids: ResolvedContextIds,
) -> Result<()> {
    let (proj, ds, vs, doc, key, seg) = filters_in;
    let filters = Filters { _proj: to_matcher(&proj), _ds: to_matcher(&ds), _vs: to_matcher(&vs), doc: to_matcher(&doc), key: to_matcher(&key), seg: to_matcher(&seg) };
    let out = File::create(file_path)?;
    let mut w = BufWriter::new(out);
    let fmt = format.unwrap_or_else(|| "kv".to_string()); // default kv

    let project = "(proj)"; let docstore = "(ds)";
    let ds_id = match ids { ResolvedContextIds::Variables { ds_id, .. } | ResolvedContextIds::Document { ds_id, .. } => ds_id };

    // VARS
    if let ResolvedContextIds::Variables { vs_id, .. } = ids {
        let vs_name = "(varstore)";
        let rows = db.list_keys(vs_id)?;
        if fmt == "kv" {
            for k in rows {
                if filters.key.as_ref().map(|m| m.is_match(&k)).unwrap_or(true) {
                    if let Some(v) = db.get_var(&k, vs_id)? { writeln!(w, "{}={}", k, v)?; }
                }
            }
        } else {
            for k in db.list_keys(vs_id)? {
                if filters.key.as_ref().map(|m| m.is_match(&k)).unwrap_or(true) {
                    if let Some(v) = db.get_var(&k, vs_id)? {
                        let row = Row::Var{ project, docstore, varstore: vs_name, key: &k, value: &v };
                        serde_json::to_writer(&mut w, &row)?; w.write_all(b"\n")?;
                    }
                }
            }
        }
    }

    // DOCS+SEGMENTS (jsonl only)
    if fmt == "jsonl" {
        let docs = db.list_docs_v2(ds_id)?;
        for d in docs.iter() {
            if filters.doc.as_ref().map(|m| m.is_match(d)).unwrap_or(true) {
                let segs = db.list_segments(d, ds_id).unwrap_or_default();
                if segs.is_empty() {
                    let row = Row::Doc{ project, docstore, doc: d };
                    serde_json::to_writer(&mut w, &row)?; w.write_all(b"\n")?;
                } else {
                    for path in segs.iter() {
                        if filters.seg.as_ref().map(|m| m.is_match(path)).unwrap_or(true) {
                            if let Some((bytes, mime)) = db.get_doc_segment(d, path, ds_id)? {
                                let row = Row::Segment{ project, docstore, doc: d, path, mime: &mime, content_b64: general_purpose::STANDARD.encode(bytes) };
                                serde_json::to_writer(&mut w, &row)?; w.write_all(b"\n")?;
                            }
                        }
                    }
                }
            }
        }
    }

    w.flush()?;
    Ok(())
}