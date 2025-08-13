use std::env;
use std::path::{PathBuf, Path};

#[derive(Debug, Clone)]
pub struct Paths {
    pub data_dir: PathBuf,          // e.g. $XDG_DATA_HOME/bookdb or ~/.local/share/bookdb
    pub config_dir: PathBuf,        // e.g. $XDG_CONFIG_HOME/bookdb or ~/.config/bookdb
    pub cursor_chain_path: PathBuf, // $config_dir/cursor.chain
    #[allow(dead_code)]
    pub cursor_base_path: PathBuf,  // $config_dir/cursor.base
    pub _base_db_path: PathBuf,     // effective base DB path (env or cursor or default)
}

fn xdg_dir(var: &str, default_suffix: &str) -> PathBuf {
    if let Ok(p) = env::var(var) {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let mut p = PathBuf::from(home);
    p.push(default_suffix);
    p
}

pub fn resolve_paths() -> Paths {
    // Data dir (override → XDG → ~/.local/share/bookdb)
    let data_dir = env::var("BOOKDB_DATA_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = xdg_dir("XDG_DATA_HOME", ".local/share");
            p.push("bookdb");
            p
        });

    // Config dir (override → XDG → ~/.config/bookdb)
    let config_dir = env::var("BOOKDB_CONFIG_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = xdg_dir("XDG_CONFIG_HOME", ".config");
            p.push("bookdb");
            p
        });

    let mut cursor_base_path = config_dir.clone();
    cursor_base_path.push("cursor.base");

    let mut cursor_chain_path = config_dir.clone();
    cursor_chain_path.push("cursor.chain");

    // Base DB path:
    // 1) BOOKDB_DB_PATH wins if set
    // 2) else cursor.base (if present and non-empty)
    // 3) else default to $data_dir/home.sqlite3
    let base_db_override = env::var("BOOKDB_DB_PATH")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let _base_db_path = base_db_override.unwrap_or_else(|| {
        if let Ok(s) = std::fs::read_to_string(&cursor_base_path) {
            let s = s.trim();
            if !s.is_empty() {
                return PathBuf::from(s);
            }
        }
        default_home_db_path(&data_dir)
    });

    Paths {
        data_dir,
        config_dir,
        cursor_chain_path,
        cursor_base_path,
        _base_db_path,
    }
}

pub fn ensure_dirs(paths: &Paths) -> std::io:: Result<(), E> {
    std::fs::create_dir_all(&paths.data_dir)?;
    std::fs::create_dir_all(&paths.config_dir)?;
    Ok(())
}

pub fn default_home_db_path(data_dir: &Path) -> PathBuf {
    let mut p = data_dir.to_path_buf();
    p.push("home.sqlite3");
    p
}
