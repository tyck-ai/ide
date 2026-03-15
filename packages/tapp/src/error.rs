use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    Init(String),
    Runtime(String),
    Serialization(String),
    Storage(String),
    Network(String),
    Permission(String),
    NotFound(String),
    InvalidState(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Init(msg) => write!(f, "Init error: {}", msg),
            Error::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Storage(msg) => write!(f, "Storage error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::Permission(msg) => write!(f, "Permission error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Runtime(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
