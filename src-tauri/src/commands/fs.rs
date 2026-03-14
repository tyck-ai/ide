use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter};

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

/// Holds the stop flag for the active watcher so we can tear it down on folder switch.
static WATCHER_STOP: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));

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
}

#[tauri::command]
pub fn watch_directory(app: AppHandle, path: String) -> Result<(), String> {
    let watch_path = PathBuf::from(&path);
    if !watch_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    // Stop any existing watcher
    {
        let mut guard = WATCHER_STOP.lock().map_err(|e| e.to_string())?;
        if let Some(old_stop) = guard.take() {
            old_stop.store(true, Ordering::Relaxed);
        }
    }

    let stop = Arc::new(AtomicBool::new(false));

    {
        let mut guard = WATCHER_STOP.lock().map_err(|e| e.to_string())?;
        *guard = Some(stop.clone());
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
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(
            &root,
            notify::RecursiveMode::Recursive,
        ) {
            eprintln!("Failed to watch directory: {}", e);
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
                        });
                    }

                    for (_key, evt) in seen {
                        let _ = app_handle.emit("fs-change", evt);
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
pub fn stop_watching() -> Result<(), String> {
    let mut guard = WATCHER_STOP.lock().map_err(|e| e.to_string())?;
    if let Some(stop) = guard.take() {
        stop.store(true, Ordering::Relaxed);
    }
    Ok(())
}
