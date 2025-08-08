// src/context.rs

use crate::db::Database;
use crate::error::{BookdbError, Result};
use crate::models::{Context, Namespace};
use log::info;
use serde_json;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use xdg::BaseDirectories;

/// The mode determines whether the resolver should create namespaces if they don't exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionMode {
    /// For read commands (`getv`, `ls`). Will fail if a namespace does not exist.
    ReadOnly,
    /// For write commands (`setv`, `import`). Will create namespaces as needed.
    GetOrCreate,
}

/// A container for the resolved database IDs.
/// This makes passing IDs around much cleaner than using tuples.
#[derive(Debug, Clone, Copy)]
pub enum ResolvedContextIds {
    Variables { p_id: i64, ds_id: i64, vs_id: i64 },
    Document { p_id: i64, ds_id: i64 },
}

/// The resolver engine. It holds a reference to the database and performs resolution.
pub struct ContextResolver<'a> {
    db: &'a Database,
}

impl<'a> ContextResolver<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// The main resolution function. It takes a context and a mode,
    /// and returns the corresponding database IDs.
    /// This is the explicit implementation of the read-only vs. write behavior.
    pub fn resolve(&self, context: &Context, mode: ResolutionMode) -> Result<ResolvedContextIds> {
        info!("Resolving context {:?} in {:?} mode", context, mode);

        match &context.active_namespace {
            Namespace::Variables { .. } => {
                let (p_id, ds_id, vs_id) = match mode {
                    ResolutionMode::ReadOnly => self.db.get_var_context_ids(context)?,
                    ResolutionMode::GetOrCreate => self.db.resolve_var_context_or_create(context)?,
                };
                Ok(ResolvedContextIds::Variables { p_id, ds_id, vs_id })
            }
            Namespace::Document => {
                let (p_id, ds_id) = match mode {
                    ResolutionMode::ReadOnly => self.db.get_doc_context_ids(context)?,
                    ResolutionMode::GetOrCreate => self.db.resolve_doc_context_or_create(context)?,
                };
                Ok(ResolvedContextIds::Document { p_id, ds_id })
            }
        }
    }
}


// --- File I/O for the cursor ---

pub fn load_or_create_context(xdg_dirs: &BaseDirectories) -> Result<Context> {
    let cursor_path = xdg_dirs.place_config_file("context.json")?;
    if !cursor_path.exists() {
        info!("Cursor file not found. Creating default at: {}", cursor_path.display());
        let context = Context::default();
        save_context(&context, &cursor_path)?;
        Ok(context)
    } else {
        let file = File::open(cursor_path)?;
        let reader = BufReader::new(file);
        let context = serde_json::from_reader(reader)?;
        Ok(context)
    }
}

pub fn save_context(context: &Context, cursor_path: &PathBuf) -> Result<()> {
    if let Some(parent) = cursor_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(cursor_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, context)?;
    Ok(())
}

// --- Parser (from previous correct version) ---
pub fn parse_context_string(s: &str) -> Result<Context> {
    let (base_name, remainder) = s.split_once('@').unwrap_or(("home", s));

    if let Some(_) = remainder.find(".var.") {
        let parts: Vec<&str> = remainder.split(".var.").collect();
        if parts.len() != 2 { return Err(BookdbError::ContextParse("Invalid variable context format.".into())); }
        
        let project_and_docstore = parts[0];
        let varstore = parts[1];

        let (project_name, docstore_name) = match project_and_docstore.rsplit_once('.') {
            Some((proj, ds)) => (proj.to_string(), ds.to_string()),
            None => (project_and_docstore.to_string(), "main".to_string()),
        };

        if varstore.is_empty() { return Err(BookdbError::ContextParse("Varstore name cannot be empty.".into())); }

        Ok(Context {
            base_name: base_name.to_string(), project_name, docstore_name,
            active_namespace: Namespace::Variables { varstore_name: varstore.to_string() },
        })

    } else if let Some(doc_pivot_index) = remainder.find(".doc.") {
        let project_and_docstore = &remainder[..doc_pivot_index];
        let (project_name, docstore_name) = match project_and_docstore.rsplit_once('.') {
            Some((proj, ds)) => (proj.to_string(), ds.to_string()),
            None => (project_and_docstore.to_string(), "main".to_string()),
        };
        Ok(Context {
            base_name: base_name.to_string(), project_name, docstore_name,
            active_namespace: Namespace::Document,
        })
    } else {
        let (project_name, docstore_name) = match remainder.rsplit_once('.') {
            Some((proj, ds)) => (proj.to_string(), ds.to_string()),
            None => (remainder.to_string(), "main".to_string()),
        };
        Ok(Context {
            base_name: base_name.to_string(), project_name, docstore_name,
            active_namespace: Namespace::Document,
        })
    }
}
