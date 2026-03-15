use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::error::{TappError, TappResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    BeforeInput,
    AfterOutput,
    OnToolCall,
    SessionStart,
    SessionEnd,
}

impl HookType {
    pub fn timeout(&self) -> Duration {
        match self {
            HookType::BeforeInput => Duration::from_millis(100),
            HookType::AfterOutput => Duration::from_millis(50),
            HookType::OnToolCall => Duration::from_millis(50),
            HookType::SessionStart => Duration::from_millis(100),
            HookType::SessionEnd => Duration::from_millis(100),
        }
    }
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

    pub fn modify_input(input: String) -> Self {
        Self {
            modified_input: Some(input),
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

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn merge(&mut self, other: HookResult) {
        if other.modified_input.is_some() {
            self.modified_input = other.modified_input;
        }
        if !other.should_continue {
            self.should_continue = false;
        }
        if other.data.is_some() {
            self.data = other.data;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistration {
    pub hook_type: HookType,
    pub handler: String,
}

#[derive(Debug, Clone)]
pub struct RegisteredHook {
    pub app_id: String,
    pub hook_type: HookType,
    pub wasm_handler: String,
    pub priority: i32,
}

pub struct HookDispatcher {
    hooks: HashMap<HookType, Vec<RegisteredHook>>,
}

impl HookDispatcher {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    pub fn register(&mut self, app_id: &str, registration: HookRegistration) {
        let hooks = self.hooks.entry(registration.hook_type).or_insert_with(Vec::new);
        // Prevent duplicate registration of the same hook from the same app
        if hooks.iter().any(|h| h.app_id == app_id && h.wasm_handler == registration.handler) {
            return;
        }
        let hook = RegisteredHook {
            app_id: app_id.to_string(),
            hook_type: registration.hook_type,
            wasm_handler: registration.handler,
            priority: 0,
        };
        hooks.push(hook);
        hooks.sort_by_key(|h| -h.priority);
    }

    pub fn unregister_app(&mut self, app_id: &str) {
        for hooks in self.hooks.values_mut() {
            hooks.retain(|h| h.app_id != app_id);
        }
    }

    pub fn get_hooks(&self, hook_type: HookType) -> Vec<&RegisteredHook> {
        self.hooks
            .get(&hook_type)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn has_hooks(&self, hook_type: HookType) -> bool {
        self.hooks
            .get(&hook_type)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

impl Default for HookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedHookDispatcher = Arc<RwLock<HookDispatcher>>;

pub fn create_shared_dispatcher() -> SharedHookDispatcher {
    Arc::new(RwLock::new(HookDispatcher::new()))
}
