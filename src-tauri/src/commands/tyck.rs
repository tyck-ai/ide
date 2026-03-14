use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use once_cell::sync::Lazy;

use super::providers;

/// Stop flags for status file watchers, keyed by watcher ID.
static WATCHER_FLAGS: Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn tyck_dir() -> PathBuf {
    PathBuf::from(home_dir()).join(".tyck")
}

fn project_name_from_cwd(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "default".to_string())
}

fn provider_dir(project: &str, provider: &str) -> PathBuf {
    tyck_dir().join("projects").join(project).join(provider)
}

pub fn session_status_path(project: &str, provider: &str, session_id: &str) -> PathBuf {
    provider_dir(project, provider)
        .join("sessions")
        .join(format!("{}.json", session_id))
}

fn statusline_script_for(project: &str, provider: &str) -> PathBuf {
    provider_dir(project, provider).join("statusline.sh")
}

// ── Commands ──

/// Initialize tyck infrastructure for a project + provider.
/// Provider-specific setup (e.g. Claude Code statusLine) is conditional.
#[tauri::command]
pub fn init_tyck(cwd: String, provider: String) -> Result<String, String> {
    let project = project_name_from_cwd(&cwd);

    // Create directory structure for all providers
    let sessions_dir = provider_dir(&project, &provider).join("sessions");
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create sessions dir: {}", e))?;

    // Claude Code-specific: statusLine script + settings.json
    if provider == "claude-code" {
        let script_path = statusline_script_for(&project, &provider);
        let script_content = format!(
            "#!/bin/bash\nOUT=\"${{TYCK_STATUS_FILE:-{}/agent-status.json}}\"\ncat > \"$OUT\"\n",
            tyck_dir().to_string_lossy()
        );
        std::fs::write(&script_path, &script_content)
            .map_err(|e| format!("Failed to write statusline script: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to chmod: {}", e))?;
        }

        let claude_settings = PathBuf::from(home_dir()).join(".claude").join("settings.json");
        let mut settings: serde_json::Value = if claude_settings.exists() {
            let content = std::fs::read_to_string(&claude_settings).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        settings["statusLine"] = serde_json::json!({
            "type": "command",
            "command": script_path.to_string_lossy()
        });
        let out = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
        std::fs::write(&claude_settings, out).map_err(|e| e.to_string())?;
    }

    Ok(project)
}

#[tauri::command]
pub fn get_session_status_path(
    cwd: String,
    provider: String,
    session_id: String,
) -> Result<String, String> {
    let project = project_name_from_cwd(&cwd);
    let path = session_status_path(&project, &provider, &session_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().to_string())
}

// ── Session listing (native discovery from agent files) ──

/// List sessions discovered from native agent files (e.g. Claude Code's ~/.claude/projects/).
#[tauri::command]
pub fn list_sessions(cwd: String) -> Vec<providers::SessionInfo> {
    providers::list_all_sessions(&cwd)
}

/// Refresh / reconcile — same as list_sessions (re-scans native files).
#[tauri::command]
pub fn reconcile_sessions(cwd: String) -> Vec<providers::SessionInfo> {
    providers::list_all_sessions(&cwd)
}

/// Get the correct resume arguments for a provider.
#[tauri::command]
pub fn get_resume_args(provider: String, session_id: String) -> Vec<String> {
    providers::resume_args(&provider, &session_id)
}

/// Prepare a session for resume by symlinking the original session file to the
/// new worktree's project directory. This is needed because Claude Code looks up
/// sessions based on the current working directory's hash.
#[tauri::command]
pub fn prepare_resume_session(
    provider: String,
    original_session_path: String,
    new_cwd: String,
    session_id: String,
) -> Result<(), String> {
    match provider.as_str() {
        "claude-code" => {
            providers::prepare_claude_resume(&original_session_path, &new_cwd, &session_id)
        }
        // Codex and other providers don't need this — they use global session storage
        _ => Ok(()),
    }
}

// ── Agent status watching ──

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub model_id: String,
    pub model_name: String,
    pub context_used_percent: f64,
    pub context_window_size: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub lines_added: u64,
    pub lines_removed: u64,
    pub session_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatusEvent {
    pub watcher_id: String,
    pub status: AgentStatus,
}

#[tauri::command]
pub async fn watch_agent_status(app: AppHandle, id: String, status_file: String) -> Result<(), String> {
    let stop = Arc::new(AtomicBool::new(false));

    {
        let mut flags = WATCHER_FLAGS.lock().map_err(|e| e.to_string())?;
        if let Some(old) = flags.get(&id) {
            old.store(true, Ordering::Relaxed);
        }
        flags.insert(id.clone(), stop.clone());
    }

    let watcher_id = id.clone();
    std::thread::spawn(move || {
        let path = PathBuf::from(&status_file);
        let mut last_modified = None;
        let mut last_content = String::new();

        loop {
            if stop.load(Ordering::Relaxed) { break; }
            std::thread::sleep(std::time::Duration::from_secs(1));
            if stop.load(Ordering::Relaxed) { break; }

            let meta = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = meta.modified().ok();
            if modified == last_modified { continue; }
            last_modified = modified;

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if content == last_content || content.trim().is_empty() { continue; }
            last_content = content.clone();

            let val: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let status = AgentStatus {
                model_id: val["model"]["id"].as_str().unwrap_or("").to_string(),
                model_name: val["model"]["display_name"].as_str().unwrap_or("").to_string(),
                context_used_percent: val["context_window"]["used_percentage"]
                    .as_f64()
                    .unwrap_or_else(|| {
                        let total = val["context_window"]["total_input_tokens"].as_f64().unwrap_or(0.0)
                            + val["context_window"]["total_output_tokens"].as_f64().unwrap_or(0.0);
                        let window = val["context_window"]["context_window_size"].as_f64().unwrap_or(1.0);
                        if window > 0.0 { (total / window) * 100.0 } else { 0.0 }
                    }),
                context_window_size: val["context_window"]["context_window_size"].as_u64().unwrap_or(0),
                total_input_tokens: val["context_window"]["total_input_tokens"].as_u64().unwrap_or(0),
                total_output_tokens: val["context_window"]["total_output_tokens"].as_u64().unwrap_or(0),
                total_cost_usd: val["cost"]["total_cost_usd"].as_f64().unwrap_or(0.0),
                lines_added: val["cost"]["total_lines_added"].as_u64().unwrap_or(0),
                lines_removed: val["cost"]["total_lines_removed"].as_u64().unwrap_or(0),
                session_id: val["session_id"].as_str().unwrap_or("").to_string(),
            };

            let _ = app.emit("agent-status", AgentStatusEvent {
                watcher_id: watcher_id.clone(),
                status,
            });
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_agent_status_watch(id: String) -> Result<(), String> {
    let mut flags = WATCHER_FLAGS.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = flags.remove(&id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}
