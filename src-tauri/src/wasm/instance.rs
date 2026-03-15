use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtxBuilder, WasiView};

use super::host::WasmHostState;
use super::limits::WasmLimits;
use super::sandbox::WasiCapabilities;
use crate::apps::error::TappError;
use crate::apps::hooks::HookRegistration;
use crate::apps::tools::ToolDefinition;
use crate::apps::ui_types::UITree;

wasmtime::component::bindgen!({
    world: "tapp-app",
    path: "../packages/tapp/wit",
});

/// Maximum size of JSON strings returned from WASM before deserialization (10 MB).
const MAX_WASM_JSON_SIZE: usize = 10 * 1024 * 1024;

fn check_json_size(json: &str, context: &str) -> Result<(), TappError> {
    if json.len() > MAX_WASM_JSON_SIZE {
        return Err(TappError::SerializationError(format!(
            "{}: response too large ({} bytes, max {})",
            context, json.len(), MAX_WASM_JSON_SIZE
        )));
    }
    Ok(())
}

pub struct WasmInstance {
    store: Store<WasmHostState>,
    bindings: TappApp,
    app_id: String,
    max_fuel: u64,
}

impl WasmInstance {
    pub fn new(
        engine: &Engine,
        component: &Component,
        capabilities: WasiCapabilities,
        app_id: String,
        limits: &WasmLimits,
    ) -> Result<Self, TappError> {
        let mut linker = Linker::new(engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| TappError::WasmInit(format!("Failed to add WASI to linker: {}", e)))?;

        let mut wasi_builder = WasiCtxBuilder::new();

        if capabilities.inherit_stdout {
            wasi_builder.inherit_stdout();
        }
        if capabilities.inherit_stderr {
            wasi_builder.inherit_stderr();
        }

        for (key, value) in &capabilities.env_vars {
            wasi_builder.env(key, value);
        }

        for path in &capabilities.fs_preopens {
            if path.exists() && !path.is_symlink() {
                let dir_perms = if capabilities.fs_writable {
                    wasmtime_wasi::DirPerms::READ | wasmtime_wasi::DirPerms::MUTATE
                } else {
                    wasmtime_wasi::DirPerms::READ
                };
                let file_perms = if capabilities.fs_writable {
                    wasmtime_wasi::FilePerms::READ | wasmtime_wasi::FilePerms::WRITE
                } else {
                    wasmtime_wasi::FilePerms::READ
                };
                wasi_builder.preopened_dir(
                    path,
                    path.to_string_lossy().to_string(),
                    dir_perms,
                    file_perms,
                ).map_err(|e| TappError::WasmInit(format!("Failed to preopen dir: {}", e)))?;
            }
        }

        let wasi_ctx = wasi_builder.build();
        let state = WasmHostState::new(wasi_ctx, ResourceTable::new(), limits.clone());
        let max_fuel = limits.max_execution_fuel;
        let mut store = Store::new(engine, state);
        store.limiter(|s| &mut s.store_limits);
        store.set_fuel(max_fuel)
            .map_err(|e| TappError::WasmInit(format!("Failed to set fuel: {}", e)))?;
        store.set_epoch_deadline(5);

        let bindings = TappApp::instantiate(&mut store, component, &linker)
            .map_err(|e| TappError::WasmInit(format!("Failed to instantiate component: {}", e)))?;

        Ok(Self { store, bindings, app_id, max_fuel })
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    fn refuel(&mut self) -> Result<(), TappError> {
        self.store.set_fuel(self.max_fuel)
            .map_err(|e| TappError::WasmCall(format!("Failed to set fuel: {}", e)))?;
        self.store.set_epoch_deadline(5);
        Ok(())
    }

    pub fn call_init(&mut self) -> Result<(), TappError> {
        self.refuel()?;
        self.bindings
            .tapp_runtime_app()
            .call_init(&mut self.store)
            .map_err(|e| TappError::WasmCall(format!("Init call failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))
    }

    pub fn call_render(&mut self) -> Result<UITree, TappError> {
        self.refuel()?;
        let json = self.bindings
            .tapp_runtime_app()
            .call_render(&mut self.store)
            .map_err(|e| TappError::WasmCall(format!("Render call failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))?;

        check_json_size(&json, "Render")?;
        serde_json::from_str(&json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse UI tree: {}", e)))
    }

    pub fn call_handle(&mut self, action: &serde_json::Value) -> Result<serde_json::Value, TappError> {
        self.refuel()?;
        let action_json = serde_json::to_string(action)
            .map_err(|e| TappError::SerializationError(e.to_string()))?;
        check_json_size(&action_json, "Handle input")?;

        let response_json = self.bindings
            .tapp_runtime_app()
            .call_handle(&mut self.store, &action_json)
            .map_err(|e| TappError::WasmCall(format!("Handle call failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))?;

        check_json_size(&response_json, "Handle")?;
        serde_json::from_str(&response_json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse response: {}", e)))
    }

    pub fn call_shutdown(&mut self) -> Result<(), TappError> {
        self.refuel()?;
        self.bindings
            .tapp_runtime_app()
            .call_shutdown(&mut self.store)
            .map_err(|e| TappError::WasmCall(format!("Shutdown call failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))
    }

    pub fn call_list_tools(&mut self) -> Result<Vec<ToolDefinition>, TappError> {
        self.refuel()?;
        let json = self.bindings
            .tapp_runtime_tools()
            .call_list_tools(&mut self.store)
            .map_err(|e| TappError::WasmCall(format!("List tools call failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))?;

        check_json_size(&json, "ListTools")?;
        serde_json::from_str(&json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse tools: {}", e)))
    }

    pub fn call_tool(&mut self, name: &str, args: &serde_json::Value) -> Result<serde_json::Value, TappError> {
        self.refuel()?;
        let args_json = serde_json::to_string(args)
            .map_err(|e| TappError::SerializationError(e.to_string()))?;
        check_json_size(&args_json, "CallTool input")?;

        let result_json = self.bindings
            .tapp_runtime_tools()
            .call_call_tool(&mut self.store, name, &args_json)
            .map_err(|e| TappError::WasmCall(format!("Call tool failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))?;

        check_json_size(&result_json, "CallTool")?;
        serde_json::from_str(&result_json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse tool result: {}", e)))
    }

    pub fn call_list_hooks(&mut self) -> Result<Vec<HookRegistration>, TappError> {
        self.refuel()?;
        let json = self.bindings
            .tapp_runtime_hooks()
            .call_list_hooks(&mut self.store)
            .map_err(|e| TappError::WasmCall(format!("List hooks call failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))?;

        check_json_size(&json, "ListHooks")?;
        serde_json::from_str(&json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse hooks: {}", e)))
    }

    pub fn invoke_hook(&mut self, hook_type: &str, data: &serde_json::Value) -> Result<serde_json::Value, TappError> {
        self.refuel()?;
        let data_json = serde_json::to_string(data)
            .map_err(|e| TappError::SerializationError(e.to_string()))?;
        check_json_size(&data_json, "InvokeHook input")?;

        let result_json = self.bindings
            .tapp_runtime_hooks()
            .call_invoke_hook(&mut self.store, hook_type, &data_json)
            .map_err(|e| TappError::WasmCall(format!("Invoke hook failed: {}", e)))?
            .map_err(|e| TappError::WasmCall(e))?;

        check_json_size(&result_json, "InvokeHook")?;
        serde_json::from_str(&result_json)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse hook result: {}", e)))
    }
}

#[derive(Clone)]
pub struct InstanceHandle {
    pub app_id: String,
    pub wasm_path: PathBuf,
    pub capabilities: WasiCapabilities,
}

pub struct InstancePool {
    handles: HashMap<String, InstanceHandle>,
}

impl InstancePool {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    pub fn get(&self, app_id: &str) -> Option<&InstanceHandle> {
        self.handles.get(app_id)
    }

    pub fn insert(&mut self, app_id: String, handle: InstanceHandle) {
        self.handles.insert(app_id, handle);
    }

    pub fn remove(&mut self, app_id: &str) -> Option<InstanceHandle> {
        self.handles.remove(app_id)
    }

    pub fn list(&self) -> Vec<String> {
        self.handles.keys().cloned().collect()
    }

    pub fn contains(&self, app_id: &str) -> bool {
        self.handles.contains_key(app_id)
    }
}

impl Default for InstancePool {
    fn default() -> Self {
        Self::new()
    }
}
