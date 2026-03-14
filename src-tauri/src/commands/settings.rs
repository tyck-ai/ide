use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

use super::terminal::resolve_binary;

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
    #[serde(default)]
    pub last_opened_folder: Option<String>,
    #[serde(default = "default_true")]
    pub review_enabled: bool,
    #[serde(default = "default_theme")]
    pub active_theme: String,
}

fn default_provider() -> String {
    "claude-code".to_string()
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "catppuccin-mocha".to_string()
}

impl Default for TyckSettings {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            last_opened_folder: None,
            review_enabled: true,
            active_theme: default_theme(),
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

#[tauri::command]
pub fn load_settings() -> TyckSettings {
    let path = settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        TyckSettings::default()
    }
}

#[tauri::command]
pub fn save_settings(settings: TyckSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
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
