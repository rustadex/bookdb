use crate::error::{Result, BookdbError};
use crate::db::Database;

#[derive(Debug, Clone, Copy)]
pub enum Namespace { Variables, Document }

#[derive(Debug, Clone)]
pub struct Context {
    pub project_name: String,
    pub docstore_name: String,
    pub active_namespace: Namespace,
    pub base_db_abs: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Prefix { Persist, Temp, ActStub }

#[derive(Debug, Clone, Copy)]
pub enum Anchor { Var, Doc }

#[derive(Debug, Clone)]
pub struct Parsed {
    #[allow(dead_code)]
    pub prefix: Prefix,
    #[allow(dead_code)]
    pub anchor: Anchor,
    pub ctx: Context,
    #[allow(dead_code)]
    pub normalized_chain: String,
    pub had_explicit_base: bool,
    pub tail: String,
}



#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ResolutionMode { ReadOnly, GetOrCreate }

#[derive(Debug, Clone, Copy)]
pub enum ResolvedContextIds {
    Variables { _project_id: i64, vs_id: i64 },
    Document  { _project_id: i64, ds_id: i64 },
}

pub fn parse_strict_fqcc(raw: &str, fallback_base_abs: &str) -> Result<Parsed> {
    if raw.is_empty() { return Err(BookdbError::ContextParse("missing context chain".into())); }
    let (prefix, rest) = match raw.chars().next().unwrap() {
        '@' => (Prefix::Persist, &raw[1..]),
        '%' => (Prefix::Temp, &raw[1..]),
        '#' => (Prefix::ActStub, &raw[1..]),
        _ => return Err(BookdbError::ContextParse("chain must start with @ or % (or # reserved)".into())),
    };

    let mut had_explicit_base = false;
    let base_and_chain = if let Some(idx) = rest.find('@') {
        had_explicit_base = true;
        (rest[..idx].to_string(), rest[idx+1..].to_string())
    } else {
        (fallback_base_abs.to_string(), rest.to_string())
    };

    let base_abs = base_and_chain.0;
    let chain_str = base_and_chain.1;
    if chain_str.is_empty() { return Err(BookdbError::ContextParse("empty context chain after base".into())); }

    let parts: Vec<&str> = chain_str.split('.').filter(|t| !t.is_empty()).collect();
    let ai = parts.iter().position(|t| t.eq_ignore_ascii_case("VAR") || t.eq_ignore_ascii_case("DOC"))
        .ok_or_else(|| BookdbError::ContextParse("anchor (VAR/DOC) required".into()))?;
    if ai < 2 { return Err(BookdbError::ContextParse("FQCC requires <PROJECT>.<DOCSTORE> before anchor".into())); }

    let project = parts[0];
    let docstore = parts[1];
    let anchor = if parts[ai].eq_ignore_ascii_case("VAR") { Anchor::Var } else { Anchor::Doc };
    let forbid = |s:&str| -> bool { let u=s.to_ascii_uppercase(); u=="VAR"||u=="DOC" };
    if forbid(project) || forbid(docstore) { return Err(BookdbError::ContextParse("project/docstore cannot be 'var' or 'doc'".into())); }

    let tail_slice = &parts[ai+1..];
    if tail_slice.is_empty() {
        return Err(BookdbError::ContextParse(match anchor {
            Anchor::Var => "VAR chain requires <VARSTORE>",
            Anchor::Doc => "DOC chain requires <DOC_KEY>",
        }.into()));
    }
    if forbid(tail_slice[0]) { return Err(BookdbError::ContextParse("varstore/doc key cannot be 'var' or 'doc'".into())); }

    let ctx = Context {
        project_name: project.to_string(),
        docstore_name: docstore.to_string(),
        active_namespace: match anchor { Anchor::Var => Namespace::Variables, Anchor::Doc => Namespace::Document },
        base_db_abs: base_abs.clone(),
    };

    Ok(Parsed {
        prefix, anchor, ctx,
        normalized_chain: chain_str.to_string(),
        had_explicit_base,
        tail: tail_slice[0].to_string(),
    })
}



pub fn resolve_ids(ctx: &Context, tail: &str, _mode: ResolutionMode, db: &Database) -> Result<ResolvedContextIds> {
    let pid = db.get_or_create_project_id(&ctx.project_name)?;
    match ctx.active_namespace {
        Namespace::Variables => {
            let vs_id = db.get_or_create_varstore_id(pid, tail)?;
            Ok(ResolvedContextIds::Variables { _project_id: pid, vs_id })
        }
        Namespace::Document => {
            let ds_id = db.get_or_create_docstore_id(pid, &ctx.docstore_name)?;
            Ok(ResolvedContextIds::Document { _project_id: pid, ds_id })
        }
    }
}

#[allow(dead_code)]
pub fn parse_chain(raw: &str) -> Result<Context> {
    Ok(parse_strict_fqcc(raw, "")?.ctx)
}
