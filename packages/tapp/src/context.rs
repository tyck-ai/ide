use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub app_id: String,
    pub data_dir: String,
    pub version: String,
}

impl Context {
    pub fn new() -> Self {
        Self {
            app_id: "unknown".to_string(),
            data_dir: ".".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
