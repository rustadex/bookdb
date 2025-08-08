// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BookdbError {
    #[error("Database Error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON Serialization/Deserialization Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Could not find required XDG directory: {0}")]
    XdgPath(String),

    #[error("Context parsing failed: {0}")]
    ContextParse(String),

    #[error("Namespace not found in database: {0}")]
    NamespaceNotFound(String),

    #[error("Key not found in database: '{0}'")]
    KeyNotFound(String),

    #[error("Invalid argument: {0}")]
    Argument(String),
}

// A type alias for a Result that uses our custom error type.
pub type Result<T> = std::result::Result<T, BookdbError>;
