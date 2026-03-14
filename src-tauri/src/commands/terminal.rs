use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use tauri::{AppHandle, Emitter};

const BACKLOG_MAX: usize = 1024 * 1024; // 1 MB ring buffer

struct PtyHandle {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
}

static PTY_HANDLES: Lazy<Mutex<HashMap<String, PtyHandle>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Per-terminal output ring buffer for replay on remount.
static PTY_BACKLOGS: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn append_backlog(id: &str, data: &[u8]) {
    if let Ok(mut backlogs) = PTY_BACKLOGS.lock() {
        let buf = backlogs.entry(id.to_string()).or_default();
        buf.extend_from_slice(data);
        // Trim from the front if over cap
        if buf.len() > BACKLOG_MAX {
            let excess = buf.len() - BACKLOG_MAX;
            buf.drain(..excess);
        }
    }
}

#[tauri::command]
pub fn get_terminal_backlog(id: String) -> Result<String, String> {
    let backlogs = PTY_BACKLOGS.lock().map_err(|e| e.to_string())?;
    match backlogs.get(&id) {
        Some(buf) => Ok(String::from_utf8_lossy(buf).to_string()),
        None => Ok(String::new()),
    }
}

#[tauri::command]
pub async fn spawn_terminal(
    app: AppHandle,
    id: String,
    cwd: Option<String>,
) -> Result<(), String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new("zsh");
    cmd.arg("-l");
    if let Some(ref dir) = cwd {
        cmd.cwd(dir);
    }
    for (key, value) in std::env::vars() {
        cmd.env(key, value);
    }

    let _child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    {
        let mut handles = PTY_HANDLES.lock().map_err(|e| e.to_string())?;
        handles.insert(
            id.clone(),
            PtyHandle {
                writer,
                master: pair.master,
            },
        );
    }

    let tid = id.clone();
    let app_clone = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    append_backlog(&tid, &buf[..n]);
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_clone.emit(&format!("pty-output-{}", tid), data);
                }
                Err(_) => break,
            }
        }
        let _ = app_clone.emit(&format!("pty-exit-{}", tid), ());
    });

    Ok(())
}

#[tauri::command]
pub fn write_terminal(id: String, data: String) -> Result<(), String> {
    let mut handles = PTY_HANDLES.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = handles.get_mut(&id) {
        handle
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| e.to_string())?;
        handle.writer.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn resize_terminal(id: String, cols: u16, rows: u16) -> Result<(), String> {
    let handles = PTY_HANDLES.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = handles.get(&id) {
        handle
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn spawn_agent_terminal(
    app: AppHandle,
    id: String,
    binary: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
    cwd: Option<String>,
) -> Result<(), String> {
    let resolved = resolve_binary(&binary);

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new(&resolved);
    for arg in &args {
        cmd.arg(arg);
    }
    if let Some(ref dir) = cwd {
        cmd.cwd(dir);
    }
    for (key, value) in std::env::vars() {
        cmd.env(key, value);
    }
    for (key, value) in &env {
        cmd.env(key, value);
    }

    let _child = pair.slave.spawn_command(cmd).map_err(|e| format!("Failed to spawn {}: {}", binary, e))?;
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    {
        let mut handles = PTY_HANDLES.lock().map_err(|e| e.to_string())?;
        handles.insert(
            id.clone(),
            PtyHandle {
                writer,
                master: pair.master,
            },
        );
    }

    let tid = id.clone();
    let app_clone = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    append_backlog(&tid, &buf[..n]);
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_clone.emit(&format!("pty-output-{}", tid), data);
                }
                Err(_) => break,
            }
        }
        let _ = app_clone.emit(&format!("pty-exit-{}", tid), ());
    });

    Ok(())
}

/// Resolve a binary name to a full path, checking common install locations
pub(crate) fn resolve_binary(name: &str) -> String {
    if std::path::Path::new(name).is_absolute() && std::path::Path::new(name).exists() {
        return name.to_string();
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<String> = vec![
        format!("/usr/local/bin/{}", name),
        format!("/opt/homebrew/bin/{}", name),
        format!("{}/.local/bin/{}", home, name),
        format!("{}/.npm/bin/{}", home, name),
        format!("{}/.claude/local/{}", home, name),
        format!("{}/.cargo/bin/{}", home, name),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
    }

    name.to_string()
}

#[tauri::command]
pub fn kill_terminal(id: String) -> Result<(), String> {
    let mut handles = PTY_HANDLES.lock().map_err(|e| e.to_string())?;
    handles.remove(&id);
    // Clean up backlog
    if let Ok(mut backlogs) = PTY_BACKLOGS.lock() {
        backlogs.remove(&id);
    }
    Ok(())
}
