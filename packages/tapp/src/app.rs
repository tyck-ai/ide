use crate::action::{Action, Response};
use crate::context::Context;
use crate::error::Result;
use crate::ui::UITree;
use crate::tools::{ToolDefinition, ToolResult};
use crate::hooks::{HookRegistration, HookResult};

pub trait App: Send + Sync {
    fn init(&mut self, ctx: &Context) -> Result<()>;

    fn shutdown(&mut self) -> Result<()>;

    fn handle(&mut self, action: Action) -> Result<Response>;

    fn render(&self) -> UITree;

    fn serialize_state(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn deserialize_state(&mut self, _state: Vec<u8>) -> Result<()> {
        Ok(())
    }
}

pub trait TappToolProvider {
    fn __tapp_list_tools() -> Vec<ToolDefinition> {
        Vec::new()
    }

    fn __tapp_call_tool(_name: &str, _args: serde_json::Value) -> ToolResult {
        ToolResult::error("No tools registered")
    }
}

pub trait TappHookProvider {
    fn __tapp_list_hooks() -> Vec<HookRegistration> {
        Vec::new()
    }

    fn __tapp_invoke_hook(_hook_type: &str, _data: &serde_json::Value) -> HookResult {
        HookResult::pass_through()
    }
}
