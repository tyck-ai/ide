use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<DirEntry>>,
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | ".git"
            | "target"
            | ".DS_Store"
            | "dist"
            | "build"
            | ".svelte-kit"
    )
}

fn read_dir_recursive(path: &Path, depth: u32) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(path) else {
        return entries;
    };

    let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    items.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_is_dir.cmp(&a_is_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for item in items {
        let name = item.file_name().to_string_lossy().to_string();
        if should_skip(&name) {
            continue;
        }
        let item_path = item.path();
        let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let children = if is_dir && depth < 10 {
            Some(read_dir_recursive(&item_path, depth + 1))
        } else {
            None
        };
        entries.push(DirEntry {
            name,
            path: item_path.to_string_lossy().to_string(),
            is_dir,
            children,
        });
    }
    entries
}

#[tauri::command]
pub fn read_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !p.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    Ok(read_dir_recursive(p, 0))
}

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {}", e))
}

// ── File system watcher ──

// Maps window_label → stop flag so each window owns its own file watcher.
static WATCHER_STOPS: Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Paths we should ignore change events from.
fn should_ignore_event(path: &Path) -> bool {
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if should_skip(&name) {
            return true;
        }
    }
    false
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsChangeEvent {
    /// The path that changed.
    pub path: String,
    /// The nearest parent directory (for targeted refresh).
    pub parent: String,
    /// Target window label — used by the frontend to discard cross-window events.
    pub window_label: String,
}

#[tauri::command]
pub fn watch_directory(app: AppHandle, path: String, window_label: String) -> Result<(), String> {
    let watch_path = PathBuf::from(&path);
    if !watch_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    // Stop any existing watcher for this window.
    {
        let mut guard = WATCHER_STOPS.lock().map_err(|e| e.to_string())?;
        if let Some(old_stop) = guard.remove(&window_label) {
            old_stop.store(true, Ordering::Relaxed);
        }
    }

    let stop = Arc::new(AtomicBool::new(false));

    {
        let mut guard = WATCHER_STOPS.lock().map_err(|e| e.to_string())?;
        guard.insert(window_label.clone(), stop.clone());
    }

    let root = watch_path.clone();
    let app_handle = app.clone();
    let stop_flag = stop.clone();

    // Spawn the watcher on a dedicated thread (notify requires it to stay alive)
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer = match new_debouncer(Duration::from_millis(300), tx) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                // Remove zombie entry so the map doesn't accumulate dead flags.
                if let Ok(mut g) = WATCHER_STOPS.lock() { g.remove(&window_label); }
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(
            &root,
            notify::RecursiveMode::Recursive,
        ) {
            eprintln!("Failed to watch directory: {}", e);
            if let Ok(mut g) = WATCHER_STOPS.lock() { g.remove(&window_label); }
            return;
        }

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Ok(events)) => {
                    // Deduplicate parent dirs to avoid flooding the frontend
                    let mut seen = HashMap::new();

                    for event in &events {
                        if event.kind != DebouncedEventKind::Any {
                            continue;
                        }

                        let event_path = &event.path;

                        if should_ignore_event(event_path) {
                            continue;
                        }

                        let parent = event_path
                            .parent()
                            .unwrap_or(event_path)
                            .to_string_lossy()
                            .to_string();

                        let path_str = event_path.to_string_lossy().to_string();

                        seen.entry(parent.clone()).or_insert_with(|| FsChangeEvent {
                            path: path_str,
                            parent,
                            window_label: window_label.clone(),
                        });
                    }

                    if let Some(win) = app_handle.get_webview_window(&window_label) {
                        for (_key, evt) in seen {
                            let _ = win.emit("fs-change", evt);
                        }
                    }
                }
                Ok(Err(err)) => {
                    eprintln!("File watcher error: {:?}", err);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check stop flag on next iteration
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }
    std::fs::File::create(&path).map_err(|e| format!("Failed to create file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn create_directory(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))
}

#[tauri::command]
pub fn rename_path(from: String, to: String) -> Result<(), String> {
    std::fs::rename(&from, &to).map_err(|e| format!("Failed to rename: {}", e))
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}

#[tauri::command]
pub fn reveal_in_file_manager(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to reveal in Finder: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| format!("Failed to reveal in Explorer: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        let parent = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path);
        std::process::Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn stop_watching(window_label: String) -> Result<(), String> {
    stop_watching_for_window(&window_label);
    Ok(())
}

/// Internal cleanup called by lib.rs on window Destroyed — no Result needed.
pub fn stop_watching_for_window(window_label: &str) {
    if let Ok(mut guard) = WATCHER_STOPS.lock() {
        if let Some(stop) = guard.remove(window_label) {
            stop.store(true, Ordering::Relaxed);
        }
    }
}
