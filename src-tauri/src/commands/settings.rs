use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Mutex;

use super::terminal::resolve_binary;

// In-memory cache of open workspace paths — populated on first load,
// kept in sync by add/remove helpers. Lets save_settings avoid a disk read.
static OPEN_WINDOWS: Mutex<Vec<String>> = Mutex::new(Vec::new());

// Full settings cache — eliminates the disk read inside persist_open_windows.
static SETTINGS_CACHE: Mutex<Option<TyckSettings>> = Mutex::new(None);

fn tyck_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".tyck")
}

fn settings_path() -> PathBuf {
    tyck_dir().join("settings.json")
}

fn themes_dir() -> PathBuf {
    tyck_dir().join("themes")
}

// ── Settings schema ──

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TyckSettings {
    #[serde(default = "default_provider")]
    pub default_provider: String,
    /// Previously `lastOpenedFolder` — kept for migration reads only; never written.
    #[serde(default, skip_serializing)]
    pub last_opened_folder: Option<String>,
    /// Workspace paths for all currently open windows. Managed by Rust only.
    #[serde(default)]
    pub open_windows: Vec<String>,
    #[serde(default = "default_workspace_mode")]
    pub workspace_mode: String,
    #[serde(default = "default_theme")]
    pub active_theme: String,
    #[serde(default)]
    pub lsp_format_on_save: Option<bool>,
    #[serde(default)]
    pub lsp_dismissed: Option<Vec<String>>,
    #[serde(default)]
    pub keybindings: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub auto_save: Option<String>,
    #[serde(default)]
    pub auto_save_delay: Option<u32>,
    #[serde(default)]
    pub inlay_hints: Option<bool>,
}

fn default_provider() -> String {
    "claude-code".to_string()
}

fn default_workspace_mode() -> String {
    "dev".to_string()
}

fn default_theme() -> String {
    "catppuccin-mocha".to_string()
}

impl Default for TyckSettings {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            last_opened_folder: None,
            open_windows: Vec::new(),
            workspace_mode: default_workspace_mode(),
            active_theme: default_theme(),
            lsp_format_on_save: None,
            lsp_dismissed: None,
            keybindings: None,
            auto_save: None,
            auto_save_delay: None,
            inlay_hints: None,
        }
    }
}

// ── Provider detection ──

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub binary: String,
    pub installed: bool,
    pub resolved_path: String,
}

/// Known providers and their binary names.
fn known_providers() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("claude-code", "Claude Code", "claude"),
        ("codex", "Codex", "codex"),
        ("cursor-agent", "Cursor", "cursor-agent"),
        ("copilot", "Copilot", "copilot"),
    ]
}

#[tauri::command]
pub fn detect_providers() -> Vec<ProviderInfo> {
    known_providers()
        .into_iter()
        .map(|(id, name, binary)| {
            let resolved = resolve_binary(binary);
            // It's "installed" if resolve_binary found an actual path (not just the bare name)
            let installed = if resolved == binary {
                // Bare name returned — wasn't found in known locations.
                // Last resort: check if `which` would find it via PATH.
                std::process::Command::new("which")
                    .arg(binary)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            } else {
                true
            };

            ProviderInfo {
                id: id.to_string(),
                display_name: name.to_string(),
                binary: binary.to_string(),
                installed,
                resolved_path: if installed { resolved } else { String::new() },
            }
        })
        .collect()
}

// ── Settings CRUD ──

pub fn load_settings_inner() -> TyckSettings {
    let path = settings_path();
    if !path.exists() {
        return TyckSettings::default();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return TyckSettings::default();
    };

    // Migrate reviewEnabled → workspaceMode.
    if json.get("reviewEnabled").is_some() && json.get("workspaceMode").is_none() {
        let mode = if json["reviewEnabled"].as_bool().unwrap_or(false) { "agent" } else { "dev" };
        json["workspaceMode"] = serde_json::Value::String(mode.to_string());
        if let Some(obj) = json.as_object_mut() {
            obj.remove("reviewEnabled");
        }
        if let Ok(migrated) = serde_json::to_string_pretty(&json) {
            let _ = std::fs::write(&path, migrated);
        }
    }

    let mut s: TyckSettings = serde_json::from_value(json).unwrap_or_default();

    // Migrate lastOpenedFolder → open_windows (one-time).
    if s.open_windows.is_empty() {
        if let Some(folder) = s.last_opened_folder.take() {
            s.open_windows.push(folder);
        }
    }

    // Populate caches so subsequent writes never need a disk read.
    *OPEN_WINDOWS.lock().unwrap() = s.open_windows.clone();
    *SETTINGS_CACHE.lock().unwrap() = Some(s.clone());

    s
}

fn save_settings_inner(settings: &TyckSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Add a workspace path to the persisted open-windows list.
pub fn add_open_window(path: &str) {
    let mut cache = OPEN_WINDOWS.lock().unwrap();
    if !cache.iter().any(|w| w == path) {
        cache.push(path.to_string());
        let windows = cache.clone();
        drop(cache);
        persist_open_windows(&windows);
    }
}

/// Remove a workspace path from the persisted open-windows list.
pub fn remove_open_window(path: &str) {
    let mut cache = OPEN_WINDOWS.lock().unwrap();
    let before = cache.len();
    cache.retain(|w| w != path);
    if cache.len() != before {
        let windows = cache.clone();
        drop(cache);
        persist_open_windows(&windows);
    }
}

/// Write open_windows into the settings file without touching user preferences.
fn persist_open_windows(windows: &[String]) {
    let mut cache = SETTINGS_CACHE.lock().unwrap();
    if let Some(ref mut s) = *cache {
        s.open_windows = windows.to_vec();
        let _ = save_settings_inner(s);
    } else {
        // Cache not yet populated (startup race) — fall back to disk read.
        drop(cache);
        let mut s = load_settings_inner();
        s.open_windows = windows.to_vec();
        let _ = save_settings_inner(&s);
    }
}

#[tauri::command]
pub fn load_settings() -> TyckSettings {
    load_settings_inner()
}

#[tauri::command]
pub fn save_settings(settings: TyckSettings) -> Result<(), String> {
    // Preserve open_windows from the in-memory cache — no disk read needed.
    let open_windows = OPEN_WINDOWS.lock().unwrap().clone();
    let to_save = TyckSettings { open_windows, ..settings };
    // Keep full settings cache in sync.
    *SETTINGS_CACHE.lock().unwrap() = Some(to_save.clone());
    save_settings_inner(&to_save)
}

// ── Custom Themes CRUD ──

#[tauri::command]
pub fn list_custom_themes() -> Vec<Value> {
    let dir = themes_dir();
    if !dir.exists() {
        return vec![];
    }

    let mut themes = vec![];
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(theme) = serde_json::from_str::<Value>(&content) {
                        themes.push(theme);
                    }
                }
            }
        }
    }
    themes
}

#[tauri::command]
pub fn save_custom_theme(theme: Value) -> Result<(), String> {
    let dir = themes_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let id = theme
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Theme must have an id")?;

    let filename = format!("{}.json", id);
    let path = dir.join(filename);
    let json = serde_json::to_string_pretty(&theme).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_custom_theme(id: String) -> Result<(), String> {
    let path = themes_dir().join(format!("{}.json", id));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn export_theme(id: String) -> Result<String, String> {
    let path = themes_dir().join(format!("{}.json", id));
    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Err("Theme not found".to_string())
    }
}
