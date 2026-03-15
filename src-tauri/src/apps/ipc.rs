use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::hooks::HookType;
use super::ui_types::{UIEvent, UITree};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppMessage {
    Init,
    Shutdown,
    UiEvent {
        component_id: String,
        event: UIEvent,
    },
    ToolCall {
        name: String,
        args: Value,
    },
    HookTrigger {
        hook: HookType,
        data: Value,
    },
    GetState,
    SetState {
        state: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppResponse {
    Ok,
    Error {
        message: String,
    },
    Render {
        tree: UITree,
    },
    ToolResult {
        success: bool,
        data: Option<Value>,
        error: Option<String>,
    },
    HookResult {
        modified_input: Option<String>,
        should_continue: bool,
        data: Option<Value>,
    },
    State {
        data: Vec<u8>,
    },
}

impl AppResponse {
    pub fn ok() -> Self {
        AppResponse::Ok
    }

    pub fn error(message: impl Into<String>) -> Self {
        AppResponse::Error {
            message: message.into(),
        }
    }

    pub fn render(tree: UITree) -> Self {
        AppResponse::Render { tree }
    }

    pub fn tool_success(data: Value) -> Self {
        AppResponse::ToolResult {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn tool_error(error: impl Into<String>) -> Self {
        AppResponse::ToolResult {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendAppEvent {
    pub app_id: String,
    pub event_type: FrontendEventType,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FrontendEventType {
    Loaded,
    Unloaded,
    Error,
    RenderUpdate,
    StateChanged,
}
