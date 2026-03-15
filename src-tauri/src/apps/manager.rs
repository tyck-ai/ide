use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use super::agent_bridge::{AgentBridge, SharedAgentBridge};
use super::error::{TappError, TappResult};
use super::hooks::{HookDispatcher, HookResult, HookType, SharedHookDispatcher};
use super::manifest::Manifest;
use super::permissions::PermissionChecker;
use super::registry::{get_apps_base_path, AppRegistry, InstalledApp};
use super::storage::AppStorage;
use super::tools::{SharedToolRegistry, ToolRegistry, ToolResult};
use super::ui_types::UITree;
use crate::wasm::host::{SharedWasmHost, WasmHost};
use crate::wasm::instance::{InstancePool, InstanceHandle, WasmInstance};
use crate::wasm::limits::WasmLimits;
use crate::wasm::sandbox::WasiCapabilities;

pub struct RunningApp {
    pub app_id: String,
    pub manifest: Manifest,
    pub storage: AppStorage,
    pub last_ui: Option<UITree>,
    pub wasm_path: PathBuf,
    pub serialized_state: Option<Vec<u8>>,
}

pub struct AppManager {
    wasm_host: SharedWasmHost,
    registry: Arc<RwLock<AppRegistry>>,
    instances: Arc<RwLock<InstancePool>>,
    live_instances: Arc<Mutex<HashMap<String, WasmInstance>>>,
    running_apps: Arc<RwLock<HashMap<String, RunningApp>>>,
    tool_registry: SharedToolRegistry,
    hook_dispatcher: SharedHookDispatcher,
    agent_bridge: SharedAgentBridge,
    hot_reload_enabled: bool,
    watch_paths: Arc<RwLock<HashMap<String, PathBuf>>>,
}

impl AppManager {
    pub async fn new() -> TappResult<Self> {
        let base_path = get_apps_base_path()?;
        std::fs::create_dir_all(&base_path)?;

        let wasm_host = Arc::new(RwLock::new(WasmHost::new(WasmLimits::default())?));
        let registry = Arc::new(RwLock::new(AppRegistry::new(&base_path)?));
        let instances = Arc::new(RwLock::new(InstancePool::new()));
        let live_instances = Arc::new(Mutex::new(HashMap::new()));
        let running_apps = Arc::new(RwLock::new(HashMap::new()));
        let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
        let hook_dispatcher = Arc::new(RwLock::new(HookDispatcher::new()));
        let agent_bridge = Arc::new(RwLock::new(AgentBridge::new()));
        let watch_paths = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            wasm_host,
            registry,
            instances,
            live_instances,
            running_apps,
            tool_registry,
            hook_dispatcher,
            agent_bridge,
            hot_reload_enabled: true,
            watch_paths,
        })
    }

    pub fn set_hot_reload(&mut self, enabled: bool) {
        self.hot_reload_enabled = enabled;
    }

    pub async fn install_app(&self, manifest_path: &Path) -> TappResult<String> {
        let manifest = Manifest::load(manifest_path)?;
        let app_id = manifest.id.clone();

        let source_dir = manifest_path.parent()
            .ok_or_else(|| TappError::ManifestError("Invalid manifest path".to_string()))?;

        let wasm_path = source_dir.join("target/wasm32-wasip2/release")
            .join(format!("{}.wasm", manifest.id.replace('-', "_")));

        if !wasm_path.exists() {
            return Err(TappError::WasmLoad(format!(
                "WASM file not found at {:?}. Did you run 'tapp build'?",
                wasm_path
            )));
        }

        let base_path = get_apps_base_path()?;
        let install_path = base_path.join(&app_id);
        std::fs::create_dir_all(&install_path)?;

        let dest_wasm = install_path.join("app.wasm");
        std::fs::copy(&wasm_path, &dest_wasm)?;

        let dest_manifest = install_path.join("manifest.json");
        std::fs::copy(manifest_path, &dest_manifest)?;

        let assets_src = source_dir.join("assets");
        if assets_src.exists() {
            let assets_dest = install_path.join("assets");
            copy_dir_recursive(&assets_src, &assets_dest)?;
        }

        let mut registry = self.registry.write().await;
        registry.install(manifest, install_path, dest_wasm)?;

        Ok(app_id)
    }

    pub async fn uninstall_app(&self, app_id: &str) -> TappResult<()> {
        if self.is_running(app_id).await {
            self.stop_app(app_id).await?;
        }

        let base_path = get_apps_base_path()?;
        let app_path = base_path.join(app_id);

        if app_path.exists() {
            std::fs::remove_dir_all(&app_path)?;
        }

        let mut registry = self.registry.write().await;
        registry.uninstall(app_id)
    }

    pub async fn start_app(&self, app_id: &str) -> TappResult<()> {
        let (manifest, wasm_path, install_path) = {
            let registry = self.registry.read().await;
            let installed = registry.get(app_id)
                .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;
            (installed.manifest.clone(), installed.wasm_path.clone(), installed.install_path.clone())
        };

        if self.is_running(app_id).await {
            return Err(TappError::AppAlreadyRunning(app_id.to_string()));
        }

        let component = {
            let mut host = self.wasm_host.write().await;
            host.load_component_cached(&wasm_path)?
        };

        let data_path = install_path.join("data");
        std::fs::create_dir_all(&data_path)?;

        // Canonicalize the data path to prevent path traversal
        let canonical_data = data_path.canonicalize()
            .map_err(|e| TappError::IoError(format!("Failed to canonicalize data path: {}", e)))?;
        let fs_writable = manifest.has_permission(&super::manifest::Permission::FsWrite)
            || manifest.has_permission(&super::manifest::Permission::FsSystem)
            || manifest.has_permission(&super::manifest::Permission::StoragePersistent);
        let mut capabilities = WasiCapabilities::for_app(app_id, canonical_data);
        capabilities.fs_writable = fs_writable;

        let (engine, limits) = {
            let host = self.wasm_host.read().await;
            (host.engine().clone(), host.limits().clone())
        };

        let mut instance = WasmInstance::new(&engine, &component, capabilities.clone(), app_id.to_string(), &limits)?;

        // Initialize the instance — if this fails, nothing has been committed yet
        instance.call_init()?;

        // Collect tools and hooks before committing to any state.
        // Only collect tools/hooks if the app has the required permissions.
        let tools = if manifest.has_permission(&super::manifest::Permission::AgentTools) {
            instance.call_list_tools().unwrap_or_default()
        } else {
            Vec::new()
        };
        let hooks = if manifest.has_permission(&super::manifest::Permission::AgentHooks) {
            instance.call_list_hooks().unwrap_or_default()
        } else {
            Vec::new()
        };

        // Only create storage if the app has storage permissions
        let storage = if manifest.has_permission(&super::manifest::Permission::StorageSession)
            || manifest.has_permission(&super::manifest::Permission::StoragePersistent)
        {
            AppStorage::new(app_id)?
        } else {
            AppStorage::empty(app_id)
        };

        // --- Commit phase: all fallible work is done, now insert into collections ---

        if !tools.is_empty() {
            let mut tool_reg = self.tool_registry.write().await;
            for tool in tools {
                if let Err(e) = tool_reg.register(app_id, tool) {
                    log::warn!("Failed to register tool: {}", e);
                }
            }
        }

        if !hooks.is_empty() {
            let mut hook_disp = self.hook_dispatcher.write().await;
            for hook in hooks {
                hook_disp.register(app_id, hook);
            }
        }

        let handle = InstanceHandle {
            app_id: app_id.to_string(),
            wasm_path: wasm_path.clone(),
            capabilities,
        };

        let running_app = RunningApp {
            app_id: app_id.to_string(),
            manifest,
            storage,
            last_ui: None,
            wasm_path,
            serialized_state: None,
        };

        {
            let mut instances = self.instances.write().await;
            instances.insert(app_id.to_string(), handle);
        }

        {
            let mut live = self.live_instances.lock().await;
            live.insert(app_id.to_string(), instance);
        }

        {
            let mut running = self.running_apps.write().await;
            running.insert(app_id.to_string(), running_app);
        }

        log::info!("Started app: {}", app_id);
        Ok(())
    }

    pub async fn stop_app(&self, app_id: &str) -> TappResult<()> {
        {
            let mut live = self.live_instances.lock().await;
            if let Some(mut instance) = live.remove(app_id) {
                if let Err(e) = instance.call_shutdown() {
                    log::warn!("Shutdown failed for {}: {}", app_id, e);
                }
            }
        }

        {
            let mut tool_reg = self.tool_registry.write().await;
            tool_reg.unregister_app(app_id);
        }

        {
            let mut hook_disp = self.hook_dispatcher.write().await;
            hook_disp.unregister_app(app_id);
        }

        {
            let mut bridge = self.agent_bridge.write().await;
            bridge.cleanup_app(app_id);
        }

        {
            let mut instances = self.instances.write().await;
            instances.remove(app_id);
        }

        {
            let mut running = self.running_apps.write().await;
            running.remove(app_id);
        }

        log::info!("Stopped app: {}", app_id);
        Ok(())
    }

    pub async fn is_running(&self, app_id: &str) -> bool {
        let running = self.running_apps.read().await;
        running.contains_key(app_id)
    }

    pub async fn get_ui(&self, app_id: &str) -> TappResult<UITree> {
        let mut live = self.live_instances.lock().await;
        let instance = live.get_mut(app_id)
            .ok_or_else(|| TappError::AppNotRunning(app_id.to_string()))?;

        instance.call_render()
    }

    pub async fn handle_action(&self, app_id: &str, action: Value) -> TappResult<Value> {
        let mut live = self.live_instances.lock().await;
        let instance = live.get_mut(app_id)
            .ok_or_else(|| TappError::AppNotRunning(app_id.to_string()))?;

        instance.call_handle(&action)
    }

    pub async fn list_installed(&self) -> Vec<InstalledApp> {
        let registry = self.registry.read().await;
        registry.list().into_iter().cloned().collect()
    }

    pub async fn list_running(&self) -> Vec<String> {
        let running = self.running_apps.read().await;
        running.keys().cloned().collect()
    }

    pub async fn get_tool_definitions(&self) -> Vec<super::tools::AgentToolDefinition> {
        let registry = self.tool_registry.read().await;
        registry.get_definitions_for_agent()
    }

    pub async fn execute_tool(&self, name: &str, args: Value) -> TappResult<ToolResult> {
        let tool = {
            let registry = self.tool_registry.read().await;
            registry.get(name).cloned()
        };

        let tool = tool.ok_or_else(|| TappError::ToolError(format!("Tool '{}' not found", name)))?;

        {
            let running = self.running_apps.read().await;
            let app = running.get(&tool.app_id)
                .ok_or_else(|| TappError::AppNotFound(tool.app_id.clone()))?;
            PermissionChecker::check(&app.manifest, &super::manifest::Permission::AgentTools)?;
        }

        let mut live = self.live_instances.lock().await;
        let instance = live.get_mut(&tool.app_id)
            .ok_or_else(|| TappError::AppNotRunning(tool.app_id.clone()))?;

        let result = instance.call_tool(&tool.definition.name, &args)?;
        
        serde_json::from_value(result)
            .map_err(|e| TappError::SerializationError(format!("Failed to parse tool result: {}", e)))
    }

    pub async fn dispatch_hook(&self, hook_type: HookType, data: Value) -> TappResult<HookResult> {
        let hooks = {
            let dispatcher = self.hook_dispatcher.read().await;
            dispatcher.get_hooks(hook_type).into_iter().cloned().collect::<Vec<_>>()
        };

        let mut result = HookResult::default();
        let timeout_duration = hook_type.timeout();

        let hook_type_str = match hook_type {
            HookType::BeforeInput => "before_input",
            HookType::AfterOutput => "after_output",
            HookType::OnToolCall => "on_tool_call",
            HookType::SessionStart => "session_start",
            HookType::SessionEnd => "session_end",
        };

        for hook in hooks {
            // Check that the app has the AgentHooks permission before invoking
            {
                let running = self.running_apps.read().await;
                if let Some(app) = running.get(&hook.app_id) {
                    PermissionChecker::check(&app.manifest, &super::manifest::Permission::AgentHooks)?;
                }
            }

            let invoke_future = async {
                let mut live = self.live_instances.lock().await;
                if let Some(instance) = live.get_mut(&hook.app_id) {
                    match instance.invoke_hook(hook_type_str, &data) {
                        Ok(result_value) => {
                            serde_json::from_value(result_value).unwrap_or_else(|e| {
                                log::warn!("Failed to deserialize hook result: {}", e);
                                HookResult::pass_through()
                            })
                        }
                        Err(e) => {
                            log::warn!("Hook invocation failed: {}", e);
                            HookResult::pass_through()
                        }
                    }
                } else {
                    HookResult::pass_through()
                }
            };

            let hook_result = match tokio::time::timeout(timeout_duration, invoke_future).await {
                Ok(hr) => hr,
                Err(_) => {
                    log::warn!(
                        "Hook {:?} for app '{}' timed out after {:?}",
                        hook_type, hook.app_id, timeout_duration
                    );
                    HookResult::pass_through()
                }
            };

            result.merge(hook_result);

            if !result.should_continue {
                break;
            }
        }

        Ok(result)
    }

    pub fn tool_registry(&self) -> SharedToolRegistry {
        self.tool_registry.clone()
    }

    pub fn hook_dispatcher(&self) -> SharedHookDispatcher {
        self.hook_dispatcher.clone()
    }

    pub fn agent_bridge(&self) -> SharedAgentBridge {
        self.agent_bridge.clone()
    }

    pub async fn hot_reload_app(&self, app_id: &str) -> TappResult<()> {
        if !self.hot_reload_enabled {
            return Err(TappError::HotReloadDisabled);
        }

        if !self.is_running(app_id).await {
            return Err(TappError::AppNotRunning(app_id.to_string()));
        }

        let wasm_path = {
            let running = self.running_apps.read().await;
            let app = running.get(app_id)
                .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;
            app.wasm_path.clone()
        };

        let serialized_state = self.serialize_app_state(app_id).await?;

        {
            let mut host = self.wasm_host.write().await;
            host.invalidate_cache(&wasm_path);
        }

        // Load and validate the NEW instance BEFORE removing the old one.
        // If the new instance fails, the old one stays intact.
        let component = {
            let mut host = self.wasm_host.write().await;
            host.load_component_cached(&wasm_path)?
        };

        let capabilities = {
            let registry = self.registry.read().await;
            let installed = registry.get(app_id)
                .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;

            let data_path = installed.install_path.join("data");
            std::fs::create_dir_all(&data_path)?;
            let canonical_data = data_path.canonicalize()
                .map_err(|e| TappError::IoError(format!("Failed to canonicalize data path: {}", e)))?;
            let fs_writable = installed.manifest.has_permission(&super::manifest::Permission::FsWrite)
                || installed.manifest.has_permission(&super::manifest::Permission::FsSystem)
                || installed.manifest.has_permission(&super::manifest::Permission::StoragePersistent);
            let mut capabilities = WasiCapabilities::for_app(app_id, canonical_data);
            capabilities.fs_writable = fs_writable;
            capabilities
        };

        let (engine, limits) = {
            let host = self.wasm_host.read().await;
            (host.engine().clone(), host.limits().clone())
        };

        let mut instance = WasmInstance::new(&engine, &component, capabilities.clone(), app_id.to_string(), &limits)?;
        instance.call_init()?;

        // New instance validated — swap atomically (consistent lock order:
        // live_instances → instances → running_apps, matching stop_app).
        let handle = InstanceHandle {
            app_id: app_id.to_string(),
            wasm_path: wasm_path.clone(),
            capabilities,
        };

        {
            let mut live = self.live_instances.lock().await;
            if let Some(mut old_instance) = live.remove(app_id) {
                let _ = old_instance.call_shutdown();
            }
            live.insert(app_id.to_string(), instance);
        }

        {
            let mut instances = self.instances.write().await;
            instances.remove(app_id);
            instances.insert(app_id.to_string(), handle);
        }

        if let Some(state) = serialized_state {
            self.deserialize_app_state(app_id, state).await?;
        }

        {
            let mut running = self.running_apps.write().await;
            if let Some(app) = running.get_mut(app_id) {
                app.serialized_state = None;
            }
        }

        log::info!("Hot reloaded app: {}", app_id);
        Ok(())
    }

    async fn serialize_app_state(&self, _app_id: &str) -> TappResult<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn deserialize_app_state(&self, _app_id: &str, _state: Vec<u8>) -> TappResult<()> {
        Ok(())
    }

    pub async fn enable_watch(&self, app_id: &str, wasm_path: PathBuf) -> TappResult<()> {
        let mut paths = self.watch_paths.write().await;
        paths.insert(app_id.to_string(), wasm_path);
        Ok(())
    }

    pub async fn disable_watch(&self, app_id: &str) -> TappResult<()> {
        let mut paths = self.watch_paths.write().await;
        paths.remove(app_id);
        Ok(())
    }

    pub async fn get_watched_apps(&self) -> Vec<(String, PathBuf)> {
        let paths = self.watch_paths.read().await;
        paths.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub async fn enable_app(&self, app_id: &str) -> TappResult<()> {
        let mut registry = self.registry.write().await;
        registry.enable(app_id)
    }

    pub async fn disable_app(&self, app_id: &str) -> TappResult<()> {
        if self.is_running(app_id).await {
            self.stop_app(app_id).await?;
        }
        let mut registry = self.registry.write().await;
        registry.disable(app_id)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> TappResult<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        // Use symlink_metadata to detect symlinks without following them
        let meta = std::fs::symlink_metadata(&src_path)?;

        // Skip symlinks to prevent path traversal attacks
        if meta.file_type().is_symlink() {
            log::warn!("Skipping symlink during install: {:?}", src_path);
            continue;
        }

        if meta.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub type SharedAppManager = Arc<RwLock<AppManager>>;

pub async fn create_shared_manager() -> TappResult<SharedAppManager> {
    let manager = AppManager::new().await?;
    Ok(Arc::new(RwLock::new(manager)))
}
