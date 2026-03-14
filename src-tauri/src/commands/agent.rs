use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;
use tokio::sync::Mutex as AsyncMutex;

type ChildHandle = Arc<AsyncMutex<Option<tokio::process::Child>>>;

static AGENT_CHILDREN: Lazy<StdMutex<HashMap<String, ChildHandle>>> =
    Lazy::new(|| StdMutex::new(HashMap::new()));

#[derive(Serialize, Clone)]
pub struct AgentEvent {
    pub session_id: String,
    pub data: serde_json::Value,
}

#[tauri::command]
pub async fn start_agent(
    app: AppHandle,
    session_id: String,
    prompt: String,
    cwd: Option<String>,
) -> Result<(), String> {
    // Resolve claude binary — macOS apps don't inherit shell PATH
    let claude_bin = which_claude().unwrap_or_else(|| "claude".into());

    let mut cmd = tokio::process::Command::new(&claude_bin);
    cmd.arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn claude ({}): {}", claude_bin, e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let child_handle: ChildHandle = Arc::new(AsyncMutex::new(Some(child)));

    {
        let mut children = AGENT_CHILDREN.lock().map_err(|e| e.to_string())?;
        children.insert(session_id.clone(), child_handle.clone());
    }

    let reader = tokio::io::BufReader::new(stdout);
    let mut lines = reader.lines();

    // Collect stderr in background
    let stderr_reader = tokio::io::BufReader::new(stderr);
    let stderr_handle = tokio::spawn(async move {
        let mut stderr_lines = stderr_reader.lines();
        let mut collected = String::new();
        while let Ok(Some(line)) = stderr_lines.next_line().await {
            if !collected.is_empty() {
                collected.push('\n');
            }
            collected.push_str(&line);
        }
        collected
    });

    let sid = session_id.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) {
                let _ = app_clone.emit(
                    "agent-event",
                    AgentEvent {
                        session_id: sid.clone(),
                        data: parsed,
                    },
                );
            }
        }

        // Wait for child to finish and get exit status
        let exit_status = {
            let mut guard = child_handle.lock().await;
            if let Some(ref mut child) = *guard {
                child.wait().await.ok()
            } else {
                None
            }
        };

        // Get stderr output
        let stderr_output = stderr_handle.await.unwrap_or_default();

        // Emit done event with exit info
        let success = exit_status.map(|s| s.success()).unwrap_or(false);
        let mut done_data = serde_json::json!({"type": "done", "success": success});
        if !stderr_output.is_empty() && !success {
            done_data["error"] = serde_json::Value::String(stderr_output);
        }

        let _ = app_clone.emit(
            "agent-event",
            AgentEvent {
                session_id: sid.clone(),
                data: done_data,
            },
        );

        // Clean up
        if let Ok(mut children) = AGENT_CHILDREN.lock() {
            children.remove(&sid);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_agent(session_id: String) -> Result<(), String> {
    let handle = {
        let mut children = AGENT_CHILDREN.lock().map_err(|e| e.to_string())?;
        children.remove(&session_id)
    };
    if let Some(handle) = handle {
        let mut guard = handle.lock().await;
        if let Some(ref mut child) = *guard {
            let _ = child.kill().await;
        }
    }
    Ok(())
}

/// Try to find the `claude` binary since macOS .app bundles have a minimal PATH
fn which_claude() -> Option<String> {
    // Common locations for claude CLI
    let candidates = [
        // If it's already in PATH
        "claude",
        // Homebrew / npm global
        "/usr/local/bin/claude",
        // Apple Silicon Homebrew
        "/opt/homebrew/bin/claude",
        // npm global (common locations)
        "/usr/local/lib/node_modules/.bin/claude",
    ];

    // Also check home dir paths
    let home = std::env::var("HOME").unwrap_or_default();
    let home_candidates = [
        format!("{}/.npm/bin/claude", home),
        format!("{}/.nvm/versions/node/default/bin/claude", home),
        format!("{}/.local/bin/claude", home),
        format!("{}/.claude/local/claude", home),
    ];

    for path in candidates {
        if std::path::Path::new(path).exists() || (path == "claude" && std::process::Command::new("which").arg("claude").output().map(|o| o.status.success()).unwrap_or(false)) {
            return Some(path.to_string());
        }
    }
    for path in home_candidates {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}
