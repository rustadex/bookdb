use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BookdbError>;

#[derive(Debug, Error)]
pub enum BookdbError {
    #[error("argument error: {0}")]
    Argument(String),
    #[error("context error: {0}")]
    ContextParse(String),
    #[error("not found: {0}")]
    KeyNotFound(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("non-numeric value: key '{0}' has value '{1}' which is not a number")]
    NonNumericValue(String, String),
    #[error("numeric overflow: operation would result in overflow")]
    NumericOverflow,
    #[error("key not found for numeric operation: {0}")]
    NumericKeyNotFound(String),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

// Added numeric error types for inc/dec operations
