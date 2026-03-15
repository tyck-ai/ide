use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::{TappError, TappResult};
use super::permissions::PermissionChecker;
use super::manifest::Permission;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub handler: String,
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
    #[serde(default)]
    pub returns: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn json(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn ok() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredTool {
    pub app_id: String,
    pub definition: ToolDefinition,
    pub wasm_handler: String,
}

pub struct ToolRegistry {
    tools: HashMap<String, RegisteredTool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, app_id: &str, definition: ToolDefinition) -> TappResult<()> {
        let full_name = format!("{}:{}", app_id, definition.name);

        if self.tools.contains_key(&full_name) {
            return Err(TappError::ToolError(format!(
                "Tool '{}' already registered",
                full_name
            )));
        }

        let handler = definition.handler.clone();
        self.tools.insert(
            full_name,
            RegisteredTool {
                app_id: app_id.to_string(),
                wasm_handler: handler,
                definition,
            },
        );

        Ok(())
    }

    pub fn unregister_app(&mut self, app_id: &str) {
        self.tools.retain(|_, tool| tool.app_id != app_id);
    }

    pub fn get(&self, name: &str) -> Option<&RegisteredTool> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<&RegisteredTool> {
        self.tools.values().collect()
    }

    pub fn list_for_app(&self, app_id: &str) -> Vec<&RegisteredTool> {
        self.tools
            .values()
            .filter(|t| t.app_id == app_id)
            .collect()
    }

    pub fn get_definitions_for_agent(&self) -> Vec<AgentToolDefinition> {
        self.tools
            .iter()
            .map(|(full_name, tool)| AgentToolDefinition {
                name: full_name.clone(),
                description: tool.definition.description.clone(),
                parameters: tool.definition.parameters.clone(),
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

pub type SharedToolRegistry = Arc<RwLock<ToolRegistry>>;

pub fn create_shared_registry() -> SharedToolRegistry {
    Arc::new(RwLock::new(ToolRegistry::new()))
}
