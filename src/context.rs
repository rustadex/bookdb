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

pub struct Parsed {
    pub ctx: Context,
    pub had_anchor: bool,
    pub persist_cursor: bool,
}

pub fn parse_chain_tokens(tokens_in: &[String]) -> Parsed {
    if tokens_in.is_empty() {
        return Parsed{ ctx: Context::default(), had_anchor:false, persist_cursor:false };
    }
    let mut tokens: Vec<String> = Vec::new();
    for t in tokens_in {
        for p in t.split('.') {
            let s = p.trim();
            if !s.is_empty() { tokens.push(s.to_string()); }
        }
    }
    let mut persist = false;
    tokens.retain(|t| {
        match t.as_str() {
            "@" => { persist = true; false },
            "%" => { persist = false; false },
            _ => true
        }
    });
    let mut last_anchor: Option<usize> = None;
    for (i,t) in tokens.iter().enumerate() {
        let tl = t.to_ascii_lowercase();
        if tl=="var" || tl=="doc" { last_anchor = Some(i); }
    }
    let had_anchor = last_anchor.is_some();
    let (project_name, docstore_name) = match last_anchor {
        Some(i) => match i {
            0 => ("GLOBAL".into(), "main".into()),
            1 => (tokens[0].clone(), "main".into()),
            _ => (tokens[0].clone(), tokens[1].clone()),
        },
        None => match tokens.len() {
            0 => ("GLOBAL".into(), "main".into()),
            1 => (tokens[0].clone(), "main".into()),
            _ => (tokens[0].clone(), tokens[1].clone()),
        }
    };
    let ns = match last_anchor {
        Some(i) => if tokens[i].eq_ignore_ascii_case("var") { Namespace::Variables } else { Namespace::Document },
        None => Namespace::Document,
    };
    Parsed { ctx: Context{ project_name, docstore_name, active_namespace: ns }, had_anchor, persist_cursor: persist }
}
