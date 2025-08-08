// src/config.rs

use crate::error::Result;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub safe_mode: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let safe_mode = match env::var("BOOKDB_SAFE_MODE") {
            Ok(val) => val.to_lowercase() != "false",
            Err(_) => true,
        };
        Ok(Config { safe_mode })
    }
}
