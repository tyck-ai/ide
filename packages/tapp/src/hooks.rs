use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    BeforeInput,
    AfterOutput,
    OnToolCall,
    SessionStart,
    SessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub modified_input: Option<String>,
    pub should_continue: bool,
    pub data: Option<Value>,
}

impl Default for HookResult {
    fn default() -> Self {
        Self {
            modified_input: None,
            should_continue: true,
            data: None,
        }
    }
}

impl HookResult {
    pub fn pass_through() -> Self {
        Self::default()
    }

    pub fn modify_input(input: impl Into<String>) -> Self {
        Self {
            modified_input: Some(input.into()),
            should_continue: true,
            data: None,
        }
    }

    pub fn cancel() -> Self {
        Self {
            modified_input: None,
            should_continue: false,
            data: None,
        }
    }

    pub fn with_data(mut self, data: impl Serialize) -> Self {
        self.data = serde_json::to_value(data).ok();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistration {
    pub hook_type: HookType,
    pub handler: String,
}
