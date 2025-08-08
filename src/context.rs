use crate::error::Result;

#[derive(Debug, Clone, Copy)]
pub enum Namespace { Variables, Document }

#[derive(Debug, Clone)]
pub struct Context {
    pub project_name: String,
    pub docstore_name: String,
    pub active_namespace: Namespace,
}

impl Context {
    pub fn default() -> Self {
        Self { project_name: "GLOBAL".into(), docstore_name: "main".into(), active_namespace: Namespace::Document }
    }
}

// Parse a chain like: "proj.ds var" or "proj.ds doc" (anchor is case-insensitive).
// If no anchor → Document.
pub fn parse_chain(chain: &str) -> Result<Context> {
    let s = chain.trim();
    if s.is_empty() { return Ok(Context::default()); }
    let tokens: Vec<&str> = s.split('.').flat_map(|p| p.split_whitespace()).collect();
    if tokens.is_empty() { return Ok(Context::default()); }

    // Find anchor "var" or "doc" (case-insensitive)
    let mut anchor_idx: Option<(usize, &str)> = None;
    for (i, t) in tokens.iter().enumerate() {
        let tl = t.to_ascii_lowercase();
        if tl=="var" || tl=="doc" { anchor_idx = Some((i, tokens[i])); break; }
    }
    match anchor_idx {
        Some((i, anchor_raw)) => {
            let anchor = anchor_raw.to_ascii_lowercase();
            let head = &tokens[..i];
            let (project_name, docstore_name) = match head.len() {
                0 => ("GLOBAL".to_string(), "main".to_string()),
                1 => (head[0].to_string(), "main".to_string()),
                _ => (head[0].to_string(), head[1].to_string()),
            };
            if anchor=="var" {
                Ok(Context{ project_name, docstore_name, active_namespace: Namespace::Variables })
            } else {
                Ok(Context{ project_name, docstore_name, active_namespace: Namespace::Document })
            }
        }
        None => {
            let (project_name, docstore_name) = match tokens.len() {
                0 => ("GLOBAL".to_string(), "main".to_string()),
                1 => (tokens[0].to_string(), "main".to_string()),
                _ => (tokens[0].to_string(), tokens[1].to_string()),
            };
            Ok(Context{ project_name, docstore_name, active_namespace: Namespace::Document })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResolutionMode { ReadOnly, GetOrCreate }

#[derive(Debug, Clone, Copy)]
pub enum ResolvedContextIds {
    Variables { vs_id: i64, ds_id: i64 },
    Document { ds_id: i64 },
}

// Resolve-or-create IDs for the context
pub fn resolve_ids(ctx: &Context, _mode: ResolutionMode, db: &crate::db::Database) -> Result<ResolvedContextIds> {
    let project_id = db.get_or_create_project(&ctx.project_name)?;
    let docstore_id = db.get_or_create_docstore(project_id, &ctx.docstore_name)?;
    let varstore_id = db.get_or_create_varstore(project_id, &ctx.docstore_name)?;
    match ctx.active_namespace {
        Namespace::Variables => Ok(ResolvedContextIds::Variables { vs_id: varstore_id, ds_id: docstore_id }),
        Namespace::Document => Ok(ResolvedContextIds::Document { ds_id: docstore_id }),
    }
}
