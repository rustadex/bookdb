// src/config.rs
use std::path::PathBuf;
use std::fs;
use std::env;

#[derive(Debug, Clone)]
pub struct Paths {
    pub home: PathBuf,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub db_path: PathBuf,
    pub cursor_base_path: PathBuf,
    pub cursor_chain_path: PathBuf,
}

fn xdg_dir(var: &str, default_suffix: &str) -> PathBuf {
    if let Ok(p) = env::var(var) {
        if !p.is_empty() { return PathBuf::from(p); }
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let mut p = PathBuf::from(home);
    p.push(default_suffix);
    p
}

pub fn resolve_paths() -> Paths {
    let data_home = xdg_dir("XDG_DATA_HOME", ".local/share");
    let config_home = xdg_dir("XDG_CONFIG_HOME", ".config");

    let mut data_dir = data_home.clone(); data_dir.push("bookdb");
    let mut config_dir = config_home.clone(); config_dir.push("bookdb");

    if let Ok(v) = env::var("BOOKDB_DATA_DIR") { if !v.is_empty() { data_dir = PathBuf::from(v); } }
    if let Ok(v) = env::var("BOOKDB_CONFIG_DIR") { if !v.is_empty() { config_dir = PathBuf::from(v); } }

    let db_path = if let Ok(v) = env::var("BOOKDB_DB_PATH") {
        if !v.is_empty() { PathBuf::from(v) } else { let mut p=data_dir.clone(); p.push("bookdb.sqlite"); p }
    } else {
        let mut p=data_dir.clone(); p.push("bookdb.sqlite"); p
    };

    let mut cursor_base_path = config_dir.clone(); cursor_base_path.push("cursor.base");
    let mut cursor_chain_path = config_dir.clone(); cursor_chain_path.push("cursor.chain");

    Paths {
        home: PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".into())),
        data_dir, config_dir, db_path, cursor_base_path, cursor_chain_path
    }
}

pub fn ensure_dirs(paths: &Paths) -> std::io::Result<()> {
    fs::create_dir_all(&paths.data_dir)?;
    fs::create_dir_all(&paths.config_dir)?;
    Ok(())
}
