use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter};

// ─────────────────────────────────────────────────────────────────────────────
// Git Directory Watcher
// ─────────────────────────────────────────────────────────────────────────────

static GIT_WATCHER_STOP: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitChangeEvent {
    pub reason: String,
}

fn is_relevant_git_path(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Watch for changes that affect git status
    path_str.contains(".git/HEAD") ||
    path_str.contains(".git/index") ||
    path_str.contains(".git/refs/") ||
    path_str.contains(".git/logs/") ||
    path_str.contains(".git/COMMIT_EDITMSG") ||
    path_str.contains(".git/MERGE_HEAD") ||
    path_str.contains(".git/REBASE_HEAD") ||
    path_str.contains(".git/stash")
}

#[tauri::command]
pub fn git_is_repo(path: String) -> bool {
    PathBuf::from(&path).join(".git").is_dir()
}

#[tauri::command]
pub fn git_init_repo(path: String) -> Result<(), String> {
    let run = |args: &[&str]| -> Result<(), String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&path)
            .output()
            .map_err(|e| format!("Failed to run git: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git {} failed: {}", args[0], stderr));
        }
        Ok(())
    };

    run(&["init"])?;
    run(&["add", "-A"])?;
    run(&["commit", "-m", "Initial commit", "--allow-empty"])?;
    Ok(())
}

/// Revert specific files on a branch to their base state (undo agent changes).
/// Used to strip rejected files before merge/push.
#[tauri::command]
pub fn git_revert_files(path: String, files: Vec<String>) -> Result<(), String> {
    for file in &files {
        let output = Command::new("git")
            .args(["checkout", "HEAD~1", "--", file])
            .current_dir(&path)
            .output()
            .map_err(|e| format!("Failed to revert {}: {}", file, e))?;

        if !output.status.success() {
            // If HEAD~1 doesn't exist (single commit), try removing the file
            let _ = Command::new("git")
                .args(["rm", "-f", "--", file])
                .current_dir(&path)
                .output();
        }
    }

    // Commit the reverts
    if !files.is_empty() {
        let _ = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&path)
            .output();
        let _ = Command::new("git")
            .args(["commit", "-m", "Revert rejected files", "--allow-empty"])
            .current_dir(&path)
            .output();
    }

    Ok(())
}

#[tauri::command]
pub fn git_has_remote(path: String) -> bool {
    Command::new("git")
        .args(["remote"])
        .current_dir(&path)
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}

#[tauri::command]
pub fn git_add_remote(path: String, name: String, url: String) -> Result<(), String> {
    let output = Command::new("git")
        .args(["remote", "add", &name, &url])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[tauri::command]
pub fn git_push_branch(path: String, branch: String) -> Result<(), String> {
    let output = Command::new("git")
        .args(["push", "--set-upstream", "origin", &branch])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("Failed to push: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Push failed: {}", stderr));
    }
    Ok(())
}

#[tauri::command]
pub fn git_merge_branch(path: String, source_branch: String, message: String) -> Result<String, String> {
    // Squash merge
    let output = Command::new("git")
        .args(["merge", "--squash", &source_branch])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("Merge failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Merge failed: {}", stderr));
    }

    // Commit
    let output = Command::new("git")
        .args(["commit", "-m", &message])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("Commit failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Commit failed: {}", stderr));
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(sha)
}

#[tauri::command]
pub fn gh_create_pr(path: String, title: String, body: String, base: String, head: String) -> Result<String, String> {
    let output = Command::new("gh")
        .args(["pr", "create", "--title", &title, "--body", &body, "--base", &base, "--head", &head])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("gh pr create failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PR creation failed: {}", stderr));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(url)
}

#[tauri::command]
pub fn watch_git_directory(app: AppHandle, path: String) -> Result<(), String> {
    let git_path = PathBuf::from(&path).join(".git");
    if !git_path.is_dir() {
        return Err("Not a git repository".to_string());
    }

    // Stop any existing git watcher
    {
        let mut guard = GIT_WATCHER_STOP.lock().map_err(|e| e.to_string())?;
        if let Some(old_stop) = guard.take() {
            old_stop.store(true, Ordering::Relaxed);
        }
    }

    let stop = Arc::new(AtomicBool::new(false));

    {
        let mut guard = GIT_WATCHER_STOP.lock().map_err(|e| e.to_string())?;
        *guard = Some(stop.clone());
    }

    let app_handle = app.clone();
    let stop_flag = stop.clone();

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer = match new_debouncer(Duration::from_millis(200), tx) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to create git watcher: {}", e);
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(
            &git_path,
            notify::RecursiveMode::Recursive,
        ) {
            eprintln!("Failed to watch .git directory: {}", e);
            return;
        }

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Ok(events)) => {
                    let mut should_emit = false;
                    let mut reason = String::from("unknown");

                    for event in &events {
                        if event.kind != DebouncedEventKind::Any {
                            continue;
                        }

                        if is_relevant_git_path(&event.path) {
                            should_emit = true;
                            let path_str = event.path.to_string_lossy();
                            if path_str.contains("index") {
                                reason = "index".to_string();
                            } else if path_str.contains("HEAD") {
                                reason = "head".to_string();
                            } else if path_str.contains("refs") {
                                reason = "refs".to_string();
                            } else if path_str.contains("stash") {
                                reason = "stash".to_string();
                            }
                            break;
                        }
                    }

                    if should_emit {
                        let _ = app_handle.emit("git-change", GitChangeEvent { reason });
                    }
                }
                Ok(Err(err)) => {
                    eprintln!("Git watcher error: {:?}", err);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_git_watching() -> Result<(), String> {
    let mut guard = GIT_WATCHER_STOP.lock().map_err(|e| e.to_string())?;
    if let Some(stop) = guard.take() {
        stop.store(true, Ordering::Relaxed);
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Data Structures
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: String,
    pub changed_files: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: char,
}

#[derive(Serialize)]
pub struct GitFullStatus {
    pub is_repo: bool,
    pub branch: String,
    pub ahead: i32,
    pub behind: i32,
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Serialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub ahead: i32,
    pub behind: i32,
    pub last_commit: String,
}

#[derive(Serialize)]
pub struct GitCommit {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub date: String,
    pub relative_date: String,
    pub parents: Vec<String>,
}

#[derive(Serialize)]
pub struct GitStash {
    pub index: usize,
    pub message: String,
    pub branch: String,
    pub date: String,
}

#[derive(Serialize)]
pub struct CommitFiles {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

#[derive(Serialize)]
pub struct CommitDetail {
    pub commit: GitCommit,
    pub files: CommitFiles,
    pub diff: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic Status (existing)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_status(path: String) -> Result<GitStatus, String> {
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&path)
        .output();

    let Ok(branch_output) = branch_output else {
        return Ok(GitStatus {
            is_repo: false,
            branch: String::new(),
            changed_files: vec![],
        });
    };

    if !branch_output.status.success() {
        return Ok(GitStatus {
            is_repo: false,
            branch: String::new(),
            changed_files: vec![],
        });
    }

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    let changed_files = String::from_utf8_lossy(&status_output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect();

    Ok(GitStatus {
        is_repo: true,
        branch,
        changed_files,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Enhanced Full Status
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_full_status(path: String) -> Result<GitFullStatus, String> {
    // Check if it's a git repo
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&path)
        .output();

    let Ok(branch_output) = branch_output else {
        return Ok(empty_full_status());
    };

    if !branch_output.status.success() {
        return Ok(empty_full_status());
    }

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    // Get ahead/behind
    let (ahead, behind) = get_ahead_behind(&path, &branch);

    // Get porcelain v2 status for detailed info
    let status_output = Command::new("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    let status_str = String::from_utf8_lossy(&status_output.stdout);
    
    let mut staged: Vec<FileStatus> = Vec::new();
    let mut unstaged: Vec<FileStatus> = Vec::new();
    let mut untracked: Vec<String> = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();

    for line in status_str.lines() {
        if line.starts_with("1 ") || line.starts_with("2 ") {
            // Changed entries: 1 XY ... path or 2 XY ... path -> orig_path
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() >= 9 {
                let xy = parts[1];
                let path_part = if line.starts_with("2 ") {
                    // Renamed: path is after tab
                    line.split('\t').nth(1).unwrap_or(parts[8])
                } else {
                    parts[8]
                };
                
                let x = xy.chars().next().unwrap_or('.');
                let y = xy.chars().nth(1).unwrap_or('.');
                
                // X = staged status, Y = unstaged status
                if x != '.' && x != '?' {
                    staged.push(FileStatus {
                        path: path_part.to_string(),
                        status: x,
                    });
                }
                if y != '.' && y != '?' {
                    unstaged.push(FileStatus {
                        path: path_part.to_string(),
                        status: y,
                    });
                }
            }
        } else if line.starts_with("? ") {
            // Untracked
            let path = line.strip_prefix("? ").unwrap_or("");
            untracked.push(path.to_string());
        } else if line.starts_with("u ") {
            // Unmerged (conflict)
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() >= 11 {
                conflicts.push(parts[10].to_string());
            }
        }
    }

    Ok(GitFullStatus {
        is_repo: true,
        branch,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
        conflicts,
    })
}

fn empty_full_status() -> GitFullStatus {
    GitFullStatus {
        is_repo: false,
        branch: String::new(),
        ahead: 0,
        behind: 0,
        staged: vec![],
        unstaged: vec![],
        untracked: vec![],
        conflicts: vec![],
    }
}

fn get_ahead_behind(path: &str, branch: &str) -> (i32, i32) {
    // Try tracking upstream first
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", &format!("{}...@{{u}}", branch)])
        .current_dir(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = s.trim().split('\t').collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                return (ahead, behind);
            }
        }
    }

    // No upstream set — count all local commits not present on any remote
    let fallback = Command::new("git")
        .args(["rev-list", "HEAD", "--not", "--remotes", "--count"])
        .current_dir(path)
        .output();

    if let Ok(output) = fallback {
        if output.status.success() {
            let count: i32 = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            return (count, 0);
        }
    }

    (0, 0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Branch Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_branches(path: String) -> Result<Vec<GitBranch>, String> {
    // Get current branch name first
    let head_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;
    
    let current_branch = String::from_utf8_lossy(&head_output.stdout).trim().to_string();

    // Use refname (full) to properly distinguish local vs remote
    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname)|%(refname:short)|%(objectname:short)|%(upstream:trackshort)|%(contents:subject)",
            "refs/heads/",
            "refs/remotes/",
        ])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let mut branches = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() >= 3 {
            let refname = parts[0];
            let name = parts[1].to_string();
            
            // Determine if remote by checking the full refname prefix
            let is_remote = refname.starts_with("refs/remotes/");
            
            // Check if this is the current branch by comparing names
            let is_current = !is_remote && name == current_branch;
            
            // Parse ahead/behind from trackshort (e.g., ">3" or "<2" or "<>")
            let track = if parts.len() > 3 { parts[3] } else { "" };
            let (ahead, behind) = parse_track_short(track);
            
            let last_commit = if parts.len() > 4 {
                parts[4].to_string()
            } else {
                String::new()
            };

            branches.push(GitBranch {
                name,
                is_current,
                is_remote,
                ahead,
                behind,
                last_commit,
            });
        }
    }

    Ok(branches)
}

fn parse_track_short(track: &str) -> (i32, i32) {
    // Format: >N (ahead), <N (behind), <>N (both)
    let mut ahead = 0;
    let mut behind = 0;
    
    if track.contains('>') {
        ahead = 1; // Just indicate we're ahead
    }
    if track.contains('<') {
        behind = 1; // Just indicate we're behind
    }
    
    (ahead, behind)
}

#[tauri::command]
pub fn git_checkout(path: String, branch: String, create: bool) -> Result<(), String> {
    let mut args = vec!["checkout"];
    if create {
        args.push("-b");
    }
    args.push(&branch);

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_create_branch(path: String, name: String, from_ref: Option<String>) -> Result<(), String> {
    let mut args = vec!["checkout", "-b", &name];
    if let Some(ref base) = from_ref {
        args.push(base);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_delete_branch(path: String, name: String, force: bool) -> Result<(), String> {
    let flag = if force { "-D" } else { "-d" };
    let output = Command::new("git")
        .args(["branch", flag, &name])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Staging Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_stage(path: String, files: Vec<String>) -> Result<(), String> {
    if files.is_empty() {
        return Ok(());
    }

    let mut args = vec!["add", "--"];
    for f in &files {
        args.push(f);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_unstage(path: String, files: Vec<String>) -> Result<(), String> {
    if files.is_empty() {
        return Ok(());
    }

    let mut args = vec!["reset", "HEAD", "--"];
    for f in &files {
        args.push(f);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_stage_all(path: String) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_discard_file(path: String, file: String) -> Result<(), String> {
    // First try checkout for tracked files
    let output = Command::new("git")
        .args(["checkout", "--", &file])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        // If file is untracked, remove it
        let rm_output = Command::new("git")
            .args(["clean", "-f", "--", &file])
            .current_dir(&path)
            .output()
            .map_err(|e| e.to_string())?;

        if !rm_output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Commit Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_commit(path: String, message: String) -> Result<String, String> {
    let output = Command::new("git")
        .args(["commit", "-m", &message])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Get the new commit SHA
    let sha_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(String::from_utf8_lossy(&sha_output.stdout).trim().to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Remote Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_push(path: String, set_upstream: bool) -> Result<(), String> {
    let mut args = vec!["push"];
    if set_upstream {
        args.extend(["--set-upstream", "origin", "HEAD"]);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_pull(path: String) -> Result<String, String> {
    let output = Command::new("git")
        .args(["pull"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub fn git_fetch(path: String) -> Result<(), String> {
    let output = Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// History Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_log(path: String, limit: Option<i32>, skip: Option<i32>) -> Result<Vec<GitCommit>, String> {
    let limit_str = limit.unwrap_or(50).to_string();
    let skip_str = skip.unwrap_or(0).to_string();

    let output = Command::new("git")
        .args([
            "log",
            &format!("-{}", limit_str),
            &format!("--skip={}", skip_str),
            "--format=%H|%h|%s|%an|%ae|%aI|%ar|%P",
        ])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let mut commits = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.splitn(8, '|').collect();
        if parts.len() >= 7 {
            let parents: Vec<String> = if parts.len() > 7 && !parts[7].is_empty() {
                parts[7].split(' ').map(|s| s.to_string()).collect()
            } else {
                vec![]
            };

            commits.push(GitCommit {
                sha: parts[0].to_string(),
                short_sha: parts[1].to_string(),
                message: parts[2].to_string(),
                author: parts[3].to_string(),
                author_email: parts[4].to_string(),
                date: parts[5].to_string(),
                relative_date: parts[6].to_string(),
                parents,
            });
        }
    }

    Ok(commits)
}

#[tauri::command]
pub fn git_show_commit(path: String, sha: String) -> Result<CommitDetail, String> {
    // Get commit info
    let commit_output = Command::new("git")
        .args([
            "show",
            "--format=%H|%h|%s|%an|%ae|%aI|%ar|%P",
            "--stat",
            &sha,
        ])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !commit_output.status.success() {
        return Err(String::from_utf8_lossy(&commit_output.stderr).to_string());
    }

    let output_str = String::from_utf8_lossy(&commit_output.stdout);
    let mut lines = output_str.lines();
    
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.splitn(8, '|').collect();
    
    if parts.len() < 7 {
        return Err("Invalid commit format".to_string());
    }

    let parents: Vec<String> = if parts.len() > 7 && !parts[7].is_empty() {
        parts[7].split(' ').map(|s| s.to_string()).collect()
    } else {
        vec![]
    };

    let commit = GitCommit {
        sha: parts[0].to_string(),
        short_sha: parts[1].to_string(),
        message: parts[2].to_string(),
        author: parts[3].to_string(),
        author_email: parts[4].to_string(),
        date: parts[5].to_string(),
        relative_date: parts[6].to_string(),
        parents,
    };

    // Get files changed
    let diff_stat = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-status", "-r", &sha])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    for line in String::from_utf8_lossy(&diff_stat.stdout).lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let status = parts[0];
            let file = parts[1].to_string();
            match status {
                "A" => added.push(file),
                "M" => modified.push(file),
                "D" => deleted.push(file),
                _ => modified.push(file),
            }
        }
    }

    // Get full diff
    let diff_output = Command::new("git")
        .args(["show", "--format=", &sha])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();

    Ok(CommitDetail {
        commit,
        files: CommitFiles { added, modified, deleted },
        diff,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Diff Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_diff_file(path: String, file: String, staged: bool) -> Result<String, String> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(&file);

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub fn git_file_content_at_head(path: String, file: String) -> Result<String, String> {
    let output = Command::new("git")
        .args(["show", &format!("HEAD:{}", file)])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        // File might be new, return empty
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Stash Operations
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn git_stash_list(path: String) -> Result<Vec<GitStash>, String> {
    let output = Command::new("git")
        .args(["stash", "list", "--format=%gd|%s|%ar"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let mut stashes = Vec::new();
    for (index, line) in String::from_utf8_lossy(&output.stdout).lines().enumerate() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() >= 3 {
            // Extract branch from message like "WIP on main: abc1234 message"
            let message = parts[1];
            let branch = if message.starts_with("WIP on ") || message.starts_with("On ") {
                message.split(':').next().unwrap_or("")
                    .trim_start_matches("WIP on ")
                    .trim_start_matches("On ")
                    .to_string()
            } else {
                String::new()
            };

            stashes.push(GitStash {
                index,
                message: message.to_string(),
                branch,
                date: parts[2].to_string(),
            });
        }
    }

    Ok(stashes)
}

#[tauri::command]
pub fn git_stash_create(path: String, message: Option<String>) -> Result<(), String> {
    let mut args = vec!["stash", "push"];
    if let Some(ref msg) = message {
        args.push("-m");
        args.push(msg);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_stash_apply(path: String, index: usize) -> Result<(), String> {
    let stash_ref = format!("stash@{{{}}}", index);
    let output = Command::new("git")
        .args(["stash", "apply", &stash_ref])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_stash_pop(path: String, index: usize) -> Result<(), String> {
    let stash_ref = format!("stash@{{{}}}", index);
    let output = Command::new("git")
        .args(["stash", "pop", &stash_ref])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn git_stash_drop(path: String, index: usize) -> Result<(), String> {
    let stash_ref = format!("stash@{{{}}}", index);
    let output = Command::new("git")
        .args(["stash", "drop", &stash_ref])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Blame Operations
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct BlameLine {
    pub sha: String,
    pub author: String,
    pub date: String,
    pub line_number: usize,
    pub content: String,
}

#[tauri::command]
pub fn git_blame_file(path: String, file: String) -> Result<Vec<BlameLine>, String> {
    let output = Command::new("git")
        .args(["blame", "--line-porcelain", &file])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let mut lines = Vec::new();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut iter = output_str.lines().peekable();
    let mut line_number = 0;

    while let Some(line) = iter.next() {
        if line.starts_with(|c: char| c.is_ascii_hexdigit()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let sha = parts.first().unwrap_or(&"").to_string();
            
            let mut author = String::new();
            let mut date = String::new();
            let mut content = String::new();

            // Parse blame block
            while let Some(prop_line) = iter.next() {
                if prop_line.starts_with("author ") {
                    author = prop_line.strip_prefix("author ").unwrap_or("").to_string();
                } else if prop_line.starts_with("author-time ") {
                    date = prop_line.strip_prefix("author-time ").unwrap_or("").to_string();
                } else if prop_line.starts_with('\t') {
                    content = prop_line.strip_prefix('\t').unwrap_or("").to_string();
                    break;
                }
            }

            line_number += 1;
            lines.push(BlameLine {
                sha: sha[..8.min(sha.len())].to_string(),
                author,
                date,
                line_number,
                content,
            });
        }
    }

    Ok(lines)
}
