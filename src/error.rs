use std::io;
use thiserror::Error;
use serde_json;

pub type Result<T> = std::result::Result<T, BookdbError>;

#[derive(Error, Debug)]
pub enum BookdbError {
    #[error("argument error: {0}")] Argument(String),
    #[error("context error: {0}")] ContextParse(String),
    #[error("not found: {0}")] KeyNotFound(String),
    #[error(transparent)] Sql(#[from] rusqlite::Error),
    #[error(transparent)] Io(#[from] io::Error),
    #[error(transparent)] Serde(#[from] serde_json::Error),
}
