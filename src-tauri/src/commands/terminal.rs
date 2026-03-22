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
    // Ensure TERM is always set — when launched as a GUI .app bundle, the parent
    // process has no TERM, causing zsh/ZLE to misbehave (garbled input, broken backspace).
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

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
        // Process exited — release the PTY master handle so the fd is closed.
        // Backlog is intentionally kept for replay until explicit kill_terminal.
        if let Ok(mut handles) = PTY_HANDLES.lock() {
            handles.remove(&tid);
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
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
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

/// Emit text directly to a terminal's event channel and backlog.
/// Used to inject synthetic progress messages before a real child process is spawned.
fn emit_pty_text(app: &AppHandle, id: &str, text: &str) {
    append_backlog(id, text.as_bytes());
    let _ = app.emit(&format!("pty-output-{}", id), text.to_string());
}

/// Non-blocking agent session spawn with Codex-style progress UI.
///
/// Creates the PTY immediately (so the terminal is visible right away), then runs
/// `git worktree add` in a background thread while streaming progress messages into
/// the terminal.  Once the worktree is ready the agent binary is spawned automatically.
///
/// The caller should have already run `prepare_agent_session` to obtain
/// `worktree_path` and `base_sha`.
#[tauri::command]
pub async fn spawn_agent_with_worktree_setup(
    app: AppHandle,
    id: String,
    cwd: String,
    worktree_path: String,
    base_sha: String,
    provider_id: Option<String>,
    binary: String,
    args: Vec<String>,
    env: HashMap<String, String>,
) -> Result<(), String> {
    let resolved = resolve_binary(&binary);

    // Open the PTY immediately — the terminal is visible before any git work starts.
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    // Destructure so slave can move into the background thread independently.
    let slave = pair.slave;
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

    // Reader thread: forwards real PTY output to the frontend.
    let tid = id.clone();
    let app_r = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    append_backlog(&tid, &buf[..n]);
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_r.emit(&format!("pty-output-{}", tid), data);
                }
            }
        }
        if let Ok(mut handles) = PTY_HANDLES.lock() {
            handles.remove(&tid);
        }
        let _ = app_r.emit(&format!("pty-exit-{}", tid), ());
    });

    // Background setup thread: holds the PTY slave until the agent is ready to start.
    let id2 = id.clone();
    let app2 = app.clone();
    std::thread::spawn(move || {
        use super::worktree::{
            copy_worktreeinclude_files, find_git, get_modified_files, get_untracked_files,
            save_worktree_meta, worktrees_dir, WorktreeInfo,
        };
        use std::path::PathBuf;

        // ── ANSI helpers ────────────────────────────────────────────────────────
        let dim_cyan = "\x1b[2m\x1b[36m";
        let reset = "\x1b[0m";
        let green = "\x1b[32m";
        let red = "\x1b[31m";

        let info = |msg: &str| {
            emit_pty_text(
                &app2,
                &id2,
                &format!("{dim_cyan}[tyck]{reset} {msg}\r\n"),
            );
        };
        let ok = |msg: &str| {
            emit_pty_text(&app2, &id2, &format!("{green}✓{reset} {msg}\r\n"));
        };
        let err = |msg: &str| {
            emit_pty_text(&app2, &id2, &format!("{red}✗{reset} {msg}\r\n"));
        };

        // ── Progress start ───────────────────────────────────────────────────────
        let _ = app2.emit(&format!("session-setup-step-{}", id2), "workspace");
        emit_pty_text(&app2, &id2, "\r\n");
        info("Creating isolated workspace...");
        let short_sha = if base_sha.len() >= 8 {
            &base_sha[..8]
        } else {
            &base_sha
        };
        info(&format!("Base commit: {short_sha}"));

        // Ensure the worktrees directory exists.
        let worktrees = worktrees_dir();
        if let Err(e) = std::fs::create_dir_all(&worktrees) {
            err(&format!("Failed to create worktrees dir: {e}"));
            log::error!("[spawn_agent_with_worktree_setup] create_dir_all failed: {e}");
            // Fall back to main cwd without worktree isolation.
            let mut cmd = portable_pty::CommandBuilder::new(&resolved);
            for arg in &args {
                cmd.arg(arg);
            }
            cmd.cwd(&cwd);
            for (key, value) in std::env::vars() {
                cmd.env(key, value);
            }
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            for (key, value) in &env {
                cmd.env(key, value);
            }
            let _ = app2.emit(&format!("session-setup-step-{}", id2), "files");
            let _ = app2.emit(&format!("session-setup-step-{}", id2), "agent");
            let _ = slave.spawn_command(cmd);
            drop(slave);
            let _ = app2.emit(&format!("session-setup-step-{}", id2), "started");
            return;
        }

        // ── git worktree add ─────────────────────────────────────────────────────
        let git = find_git();
        let wt_path = PathBuf::from(&worktree_path);
        info(&format!(
            "git worktree add --detach {} {short_sha}",
            wt_path.display()
        ));

        // Pipe stderr so we can stream git's progress output to the overlay.
        let spawn_result = std::process::Command::new(&git)
            .args([
                "worktree",
                "add",
                "--detach",
                &wt_path.to_string_lossy(),
                &base_sha,
            ])
            .current_dir(&cwd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let worktree_ok = match spawn_result {
            Err(e) => {
                err(&format!("git worktree add failed to start: {e}"));
                log::error!("[spawn_agent_with_worktree_setup] git spawn error: {e}");
                false
            }
            Ok(mut child) => {
                // Stream every stderr line to the frontend as a progress event.
                // git writes progress with \r (in-place) when stderr is a TTY; when
                // piped it may suppress progress, but we forward whatever it sends.
                if let Some(stderr) = child.stderr.take() {
                    let id_p = id2.clone();
                    let app_p = app2.clone();
                    std::thread::spawn(move || {
                        use std::io::Read;
                        let mut reader = std::io::BufReader::new(stderr);
                        let mut buf = vec![0u8; 256];
                        let mut leftover = String::new();
                        loop {
                            match reader.read(&mut buf) {
                                Ok(0) | Err(_) => break,
                                Ok(n) => {
                                    leftover
                                        .push_str(&String::from_utf8_lossy(&buf[..n]));
                                    // Split on \r or \n — git uses both
                                    while let Some(pos) =
                                        leftover.find(|c: char| c == '\r' || c == '\n')
                                    {
                                        let segment =
                                            leftover[..pos].trim().to_string();
                                        leftover = leftover[pos + 1..].to_string();
                                        if !segment.is_empty() {
                                            let _ = app_p.emit(
                                                &format!(
                                                    "session-setup-progress-{}",
                                                    id_p
                                                ),
                                                &segment,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    });
                }

                match child.wait() {
                    Err(e) => {
                        err(&format!("git worktree add failed: {e}"));
                        log::error!(
                            "[spawn_agent_with_worktree_setup] git wait error: {e}"
                        );
                        false
                    }
                    Ok(status) if !status.success() => {
                        err("git worktree add failed");
                        log::error!(
                            "[spawn_agent_with_worktree_setup] git exited non-zero"
                        );
                        false
                    }
                    Ok(_) if !wt_path.exists() => {
                        err("Worktree directory was not created");
                        log::error!(
                            "[spawn_agent_with_worktree_setup] git ok but dir missing: {}",
                            wt_path.display()
                        );
                        false
                    }
                    Ok(_) => {
                        ok("Workspace ready");
                        log::info!(
                            "[spawn_agent_with_worktree_setup] worktree created at '{}'",
                            wt_path.display()
                        );
                        true
                    }
                }
            }
        };

        let agent_cwd = if worktree_ok {
            worktree_path.clone()
        } else {
            cwd.clone()
        };

        // Advance overlay to "files" step regardless of git outcome.
        let _ = app2.emit(&format!("session-setup-step-{}", id2), "files");

        // ── File copy + metadata ─────────────────────────────────────────────────
        if worktree_ok {
            info("Copying project files...");
            let _ = copy_worktreeinclude_files(&cwd, &worktree_path);

            // Collect untracked/modified lists for metadata (git ls-files may be slow
            // on large repos, but we're already in a background thread so it's fine).
            let initial_untracked = get_untracked_files(&cwd);
            let initial_modified = get_modified_files(&cwd);

            // Copy untracked files into the worktree (reuse the list we just fetched).
            for file in &initial_untracked {
                let src = std::path::Path::new(&cwd).join(file);
                let dst = std::path::Path::new(&worktree_path).join(file);
                if src.is_file() {
                    if let Some(parent) = dst.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::copy(&src, &dst);
                }
            }

            let now = chrono::Utc::now().to_rfc3339();
            let wt_info = WorktreeInfo {
                session_id: id2.clone(),
                base_sha: base_sha.clone(),
                worktree_path: worktree_path.clone(),
                main_cwd: cwd.clone(),
                created_at: now,
                initial_untracked,
                initial_modified,
                provider_id,
                provider_session_id: None,
                claude_session_id: None,
            };
            if let Err(e) = save_worktree_meta(&wt_info) {
                log::warn!("[spawn_agent_with_worktree_setup] save_worktree_meta failed: {e}");
            }

            // Signal the frontend that worktree is ready (triggers refreshDiffs).
            let _ = app2.emit(&format!("worktree-ready-{id2}"), ());
            ok("Project files ready");
        }

        // ── Spawn agent ──────────────────────────────────────────────────────────
        let _ = app2.emit(&format!("session-setup-step-{}", id2), "agent");
        info(&format!("Starting {binary}..."));
        emit_pty_text(&app2, &id2, "\r\n");

        let mut cmd = portable_pty::CommandBuilder::new(&resolved);
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.cwd(&agent_cwd);
        for (key, value) in std::env::vars() {
            cmd.env(key, value);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        for (key, value) in &env {
            cmd.env(key, value);
        }

        match slave.spawn_command(cmd) {
            Ok(_child) => {
                drop(slave);
                log::info!(
                    "[spawn_agent_with_worktree_setup] '{}' started in '{}'",
                    binary,
                    agent_cwd
                );
                let _ = app2.emit(&format!("session-setup-step-{}", id2), "started");
            }
            Err(e) => {
                err(&format!("Failed to start {binary}: {e}"));
                log::error!("[spawn_agent_with_worktree_setup] spawn failed: {e}");
                drop(slave);
                let _ = app2.emit(&format!("session-setup-step-{}", id2), "started");
            }
        }
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
        format!("{}/.claude/local/{}", home, name),
        format!("{}/.cargo/bin/{}", home, name),
        // Volta — static shim directory, works without sourcing any shell config
        format!("{}/.volta/bin/{}", home, name),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            log::info!("[resolve_binary] '{}' found at {}", name, path);
            return path.clone();
        }
    }

    // nvm — node versions live at ~/.nvm/versions/node/<ver>/bin/.
    // nvm only adds to PATH in ~/.zshrc (interactive shells), so login-shell
    // PATH bootstrapping misses it. Scan all installed versions, preferring newer.
    let nvm_versions = format!("{}/.nvm/versions/node", home);
    match std::fs::read_dir(&nvm_versions) {
        Ok(entries) => {
            let mut versions: Vec<_> = entries.flatten().collect();
            versions.sort_by(|a, b| b.file_name().cmp(&a.file_name())); // newest first
            let version_names: Vec<String> = versions.iter().map(|e| e.file_name().to_string_lossy().to_string()).collect();
            log::info!("[resolve_binary] '{}' not in standard paths; scanning nvm versions: [{}]", name, version_names.join(", "));
            for entry in versions {
                let candidate = entry.path().join("bin").join(name);
                if candidate.exists() {
                    let found = candidate.to_string_lossy().to_string();
                    log::info!("[resolve_binary] '{}' found via nvm: {}", name, found);
                    return found;
                }
            }
        }
        Err(_) => {
            log::info!("[resolve_binary] nvm not found at {}", nvm_versions);
        }
    }

    // fnm — similar to nvm but different dir layout
    let fnm_dir = format!("{}/.local/share/fnm/node-versions", home);
    match std::fs::read_dir(&fnm_dir) {
        Ok(entries) => {
            let mut versions: Vec<_> = entries.flatten().collect();
            versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            let version_names: Vec<String> = versions.iter().map(|e| e.file_name().to_string_lossy().to_string()).collect();
            log::info!("[resolve_binary] '{}' not in nvm; scanning fnm versions: [{}]", name, version_names.join(", "));
            for entry in versions {
                let candidate = entry.path().join("installation").join("bin").join(name);
                if candidate.exists() {
                    let found = candidate.to_string_lossy().to_string();
                    log::info!("[resolve_binary] '{}' found via fnm: {}", name, found);
                    return found;
                }
            }
        }
        Err(_) => {
            log::info!("[resolve_binary] fnm not found at {}", fnm_dir);
        }
    }

    log::warn!("[resolve_binary] '{}' not found in any known location", name);
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
