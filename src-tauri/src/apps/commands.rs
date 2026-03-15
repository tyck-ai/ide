use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use super::error::TappResult;
use super::hooks::HookType;
use super::manager::{AppManager, SharedAppManager};
use super::registry::InstalledApp;
use super::tools::AgentToolDefinition;
use super::store::{AppListing, AppStore, UpdateInfo};
use super::ui_types::UITree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub running: bool,
    pub layout: String,
}

impl AppInfo {
    pub fn from_installed(app: &InstalledApp, running: bool) -> Self {
        Self {
            id: app.id.clone(),
            name: app.manifest.name.clone(),
            version: app.manifest.version.clone(),
            description: app.manifest.description.clone(),
            enabled: app.enabled,
            running,
            layout: format!("{:?}", app.manifest.ui.layout).to_lowercase(),
        }
    }
}

#[tauri::command]
pub async fn tapp_list_apps(
    manager: State<'_, SharedAppManager>,
) -> Result<Vec<AppInfo>, String> {
    let manager = manager.read().await;
    let installed = manager.list_installed().await;
    let running = manager.list_running().await;

    let apps = installed
        .into_iter()
        .map(|app| {
            let is_running = running.contains(&app.id);
            AppInfo::from_installed(&app, is_running)
        })
        .collect();

    Ok(apps)
}

#[tauri::command]
pub async fn tapp_install_app(
    manifest_path: String,
    manager: State<'_, SharedAppManager>,
) -> Result<String, String> {
    let manager = manager.read().await;
    let path = PathBuf::from(manifest_path);
    manager.install_app(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_uninstall_app(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.uninstall_app(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_start_app(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.start_app(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_stop_app(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.stop_app(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_get_tool_definitions(
    manager: State<'_, SharedAppManager>,
) -> Result<Vec<AgentToolDefinition>, String> {
    let manager = manager.read().await;
    Ok(manager.get_tool_definitions().await)
}

#[tauri::command]
pub async fn tapp_execute_tool(
    name: String,
    args: Value,
    manager: State<'_, SharedAppManager>,
) -> Result<Value, String> {
    let manager = manager.read().await;
    let result = manager.execute_tool(&name, args).await.map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_dispatch_hook(
    hook_type: HookType,
    data: Value,
    manager: State<'_, SharedAppManager>,
) -> Result<Value, String> {
    let manager = manager.read().await;
    let result = manager.dispatch_hook(hook_type, data).await.map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_hot_reload(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.hot_reload_app(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_enable_watch(
    app_id: String,
    wasm_path: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    let path = PathBuf::from(wasm_path);
    manager.enable_watch(&app_id, path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_disable_watch(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.disable_watch(&app_id).await.map_err(|e| e.to_string())
}

pub type SharedAppStore = Arc<RwLock<AppStore>>;

#[tauri::command]
pub async fn tapp_store_search(
    query: String,
    store: State<'_, SharedAppStore>,
) -> Result<Vec<AppListing>, String> {
    let store = store.read().await;
    store.search(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_store_get_listing(
    app_id: String,
    store: State<'_, SharedAppStore>,
) -> Result<Option<AppListing>, String> {
    let store = store.read().await;
    store.get_listing(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_store_refresh(
    store: State<'_, SharedAppStore>,
) -> Result<(), String> {
    let mut store = store.write().await;
    store.refresh_listings().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_store_check_updates(
    manager: State<'_, SharedAppManager>,
    store: State<'_, SharedAppStore>,
) -> Result<Vec<UpdateInfo>, String> {
    let manager = manager.read().await;
    let store = store.read().await;
    
    let installed_apps: Vec<(String, String)> = manager.list_installed().await
        .into_iter()
        .map(|app| (app.id, app.manifest.version))
        .collect();
    
    store.check_updates(&installed_apps).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_store_download(
    app_id: String,
    store: State<'_, SharedAppStore>,
) -> Result<String, String> {
    let store = store.read().await;
    let path = store.download_app(&app_id).await.map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn tapp_enable_app(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.enable_app(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_disable_app(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<(), String> {
    let manager = manager.read().await;
    manager.disable_app(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_get_ui(
    app_id: String,
    manager: State<'_, SharedAppManager>,
) -> Result<UITree, String> {
    let manager = manager.read().await;
    manager.get_ui(&app_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tapp_dispatch_action(
    app_id: String,
    action: Value,
    manager: State<'_, SharedAppManager>,
) -> Result<Value, String> {
    let manager = manager.read().await;
    manager.handle_action(&app_id, action).await.map_err(|e| e.to_string())
}

pub fn get_commands() -> Vec<&'static str> {
    vec![
        "tapp_list_apps",
        "tapp_install_app",
        "tapp_uninstall_app",
        "tapp_start_app",
        "tapp_stop_app",
        "tapp_enable_app",
        "tapp_disable_app",
        "tapp_get_tool_definitions",
        "tapp_execute_tool",
        "tapp_dispatch_hook",
        "tapp_hot_reload",
        "tapp_enable_watch",
        "tapp_disable_watch",
        "tapp_store_search",
        "tapp_store_get_listing",
        "tapp_store_refresh",
        "tapp_store_check_updates",
        "tapp_store_download",
        "tapp_get_ui",
        "tapp_dispatch_action",
    ]
}
