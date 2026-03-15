use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum TappError {
    #[error("WASM initialization failed: {0}")]
    WasmInit(String),

    #[error("WASM module load failed: {0}")]
    WasmLoad(String),

    #[error("WASM function call failed: {0}")]
    WasmCall(String),

    #[error("App not found: {0}")]
    AppNotFound(String),

    #[error("App already running: {0}")]
    AppAlreadyRunning(String),

    #[error("App not running: {0}")]
    AppNotRunning(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Manifest error: {0}")]
    ManifestError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Hook timeout: {0}")]
    HookTimeout(String),

    #[error("Tool execution failed: {0}")]
    ToolError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Hot reload is disabled")]
    HotReloadDisabled,

    #[error("State serialization failed: {0}")]
    StateSerializationError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Package signing error: {0}")]
    SigningError(String),

    #[error("Package verification failed: {0}")]
    VerificationError(String),

    #[error("Update check failed: {0}")]
    UpdateCheckError(String),

    #[error("Download failed: {0}")]
    DownloadError(String),
}

impl From<std::io::Error> for TappError {
    fn from(e: std::io::Error) -> Self {
        TappError::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for TappError {
    fn from(e: serde_json::Error) -> Self {
        TappError::SerializationError(e.to_string())
    }
}

impl From<rusqlite::Error> for TappError {
    fn from(e: rusqlite::Error) -> Self {
        TappError::StorageError(e.to_string())
    }
}

pub type TappResult<T> = Result<T, TappError>;
