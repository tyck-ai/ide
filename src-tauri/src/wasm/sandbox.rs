use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasiCapabilities {
    pub fs_preopens: Vec<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub inherit_stdout: bool,
    pub inherit_stderr: bool,
    /// Reserved for future use — not yet enforced at the WASI level.
    pub allow_network: bool,
    pub fs_writable: bool,
}

impl WasiCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_preopen(mut self, path: PathBuf) -> Self {
        self.fs_preopens.push(path);
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    pub fn with_stdout(mut self) -> Self {
        self.inherit_stdout = true;
        self
    }

    pub fn with_stderr(mut self) -> Self {
        self.inherit_stderr = true;
        self
    }

    pub fn with_network(mut self) -> Self {
        self.allow_network = true;
        self
    }

    pub fn for_app(app_id: &str, data_dir: PathBuf) -> Self {
        Self::new()
            .with_preopen(data_dir)
            .with_env("TAPP_APP_ID", app_id)
            .with_stdout()
            .with_stderr()
    }
}
