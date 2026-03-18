use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Atomic counter for unique temp file names to prevent race conditions
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Mutex to prevent concurrent worktree creation for the same session
static WORKTREE_CREATION_LOCK: Mutex<()> = Mutex::new(());

/// Map of active discovery tasks (session_id -> should_stop flag)
static DISCOVERY_TASKS: Mutex<Option<HashMap<String, std::sync::Arc<AtomicBool>>>> = Mutex::new(None);

fn get_discovery_tasks() -> std::sync::MutexGuard<'static, Option<HashMap<String, std::sync::Arc<AtomicBool>>>> {
    // Recover from poisoned lock rather than panicking
    let mut guard = DISCOVERY_TASKS.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    guard
}

/// Generate a unique temp file prefix for merge operations
fn unique_temp_prefix() -> String {
    let id = TEMP_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("tyck_{}_{}", std::process::id(), id)
}

/// Minimum git version required for reliable worktree support
const MIN_GIT_VERSION: (u32, u32) = (2, 17);

/// Maximum age in days before a worktree is considered stale
const STALE_WORKTREE_DAYS: i64 = 7;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub session_id: String,
    /// The base commit SHA (may be a stash commit if there were uncommitted changes)
    pub base_sha: String,
    pub worktree_path: String,
    pub main_cwd: String,
    pub created_at: String,
    /// Files that were untracked in main when the worktree was created.
    /// These are excluded from diff scans to avoid false positives.
    #[serde(default)]
    pub initial_untracked: Vec<String>,
    /// Files that were modified (uncommitted) in main when the worktree was created.
    /// These are excluded from diff scans to avoid false positives.
    #[serde(default)]
    pub initial_modified: Vec<String>,
    /// Provider ID (e.g., "claude-code", "codex") that created this worktree.
    #[serde(default)]
    pub provider_id: Option<String>,
    /// Provider's session ID for this worktree (if discovered).
    /// Used to map history entries back to their worktree on resume.
    #[serde(default)]
    pub provider_session_id: Option<String>,
    /// Legacy field - kept for backward compatibility with existing worktree metadata.
    #[serde(default, alias = "claudeSessionId")]
    claude_session_id: Option<String>,
}

impl WorktreeInfo {
    /// Get the provider session ID, checking the new field first, then legacy.
    pub fn get_provider_session_id(&self) -> Option<&str> {
        self.provider_session_id.as_deref()
            .or(self.claude_session_id.as_deref())
    }
    
    /// Set the provider session ID (writes to new field only).
    pub fn set_provider_session_id(&mut self, session_id: String) {
        self.provider_session_id = Some(session_id);
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeFileDiff {
    pub path: String,
    pub status: String, // A, M, D
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AcceptResult {
    pub success: bool,
    pub conflict: bool,
    pub conflict_reason: Option<String>,
    /// True if conflict markers were written to the worktree file
    pub conflict_written: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionReviewRecord {
    pub session_id: String,
    pub main_cwd: String,
    pub created_at: String,
    pub completed_at: String,
    pub files: Vec<ReviewedFileRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReviewedFileRecord {
    pub path: String,
    pub status: String,
    pub decision: String,
}

fn tyck_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".tyck")
}

fn worktrees_dir() -> PathBuf {
    tyck_home().join("worktrees")
}

fn worktree_meta_path(session_id: &str) -> PathBuf {
    worktrees_dir().join(format!("{}.json", session_id))
}

fn worktree_path(session_id: &str) -> PathBuf {
    worktrees_dir().join(session_id)
}

fn reviews_dir() -> PathBuf {
    tyck_home().join("reviews")
}

/// Validate that session_id is safe to use as a path component.
fn validate_session_id(session_id: &str) -> Result<(), String> {
    if session_id.is_empty() {
        return Err("Session ID cannot be empty".to_string());
    }
    if !session_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err("Session ID must contain only alphanumeric characters and hyphens".to_string());
    }
    if session_id.len() > 64 {
        return Err("Session ID too long".to_string());
    }
    Ok(())
}

/// Validate that a file path is safe (no path traversal attacks).
fn validate_file_path(file_path: &str) -> Result<(), String> {
    if file_path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }
    // Check for path traversal attempts
    if file_path.contains("..") {
        return Err("File path cannot contain '..' (path traversal not allowed)".to_string());
    }
    // Check for absolute paths (should be relative to worktree/main)
    if file_path.starts_with('/') || file_path.starts_with('\\') {
        return Err("File path must be relative, not absolute".to_string());
    }
    // Check for Windows-style absolute paths (e.g., C:\)
    if file_path.len() >= 2 && file_path.chars().nth(1) == Some(':') {
        return Err("File path must be relative, not absolute".to_string());
    }
    Ok(())
}

/// Check if the git version meets minimum requirements.
#[tauri::command]
pub fn check_git_version() -> Result<bool, String> {
    let output = Command::new("git")
        .args(["--version"])
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        return Err("git --version failed".to_string());
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    // Parse "git version X.Y.Z" or "git version X.Y.Z.windows.N"
    let parts: Vec<&str> = version_str.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("Could not parse git version".to_string());
    }

    let version_parts: Vec<&str> = parts[2].split('.').collect();
    if version_parts.len() < 2 {
        return Err("Could not parse git version numbers".to_string());
    }

    let major: u32 = version_parts[0].parse().unwrap_or(0);
    let minor: u32 = version_parts[1].parse().unwrap_or(0);

    Ok(major > MIN_GIT_VERSION.0
        || (major == MIN_GIT_VERSION.0 && minor >= MIN_GIT_VERSION.1))
}

/// Check if a path is inside a git worktree (not the main repo).
fn is_inside_worktree(path: &Path) -> bool {
    let dot_git = path.join(".git");
    dot_git.is_file()
}

/// Check if a directory is a valid git repository.
fn is_git_repo(cwd: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the list of untracked files (not ignored) in a git repo.
fn get_untracked_files(cwd: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect(),
        _ => vec![],
    }
}

/// Get the list of modified (uncommitted) files in a git repo.
fn get_modified_files(cwd: &str) -> Vec<String> {
    let mut files = Vec::new();

    // Staged changes
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).trim().split('\n') {
                if !line.is_empty() && !files.contains(&line.to_string()) {
                    files.push(line.to_string());
                }
            }
        }
    }

    // Unstaged changes
    if let Ok(output) = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).trim().split('\n') {
                if !line.is_empty() && !files.contains(&line.to_string()) {
                    files.push(line.to_string());
                }
            }
        }
    }

    files
}

fn save_worktree_meta(info: &WorktreeInfo) -> Result<(), String> {
    let dir = worktrees_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(info).map_err(|e| e.to_string())?;
    fs::write(worktree_meta_path(&info.session_id), json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_worktree_meta(session_id: &str) -> Result<WorktreeInfo, String> {
    let path = worktree_meta_path(session_id);
    let data = fs::read_to_string(&path)
        .map_err(|_| format!("Worktree meta for session {} not found", session_id))?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

/// Create a git worktree for an agent session.
/// The worktree is detached and captures the exact state of the main repo,
/// including uncommitted changes, to ensure diffs are accurate.
#[tauri::command]
pub fn create_worktree(cwd: String, session_id: String, provider_id: Option<String>) -> Result<WorktreeInfo, String> {
    // Validate session_id
    validate_session_id(&session_id)?;

    // Acquire lock to prevent concurrent creation of worktrees
    let _lock = WORKTREE_CREATION_LOCK
        .lock()
        .map_err(|_| "Failed to acquire worktree creation lock")?;

    // Check if worktree already exists for this session
    let wt_path = worktree_path(&session_id);
    if wt_path.exists() {
        // If metadata exists too, return the existing info
        if let Ok(info) = load_worktree_meta(&session_id) {
            return Ok(info);
        }
        // Worktree dir exists but no metadata - clean it up first
        let _ = fs::remove_dir_all(&wt_path);
    }

    // Check if cwd is already a worktree (prevent nested worktrees)
    if is_inside_worktree(Path::new(&cwd)) {
        return Err(
            "Cannot create worktree from inside another worktree. Use the main repository."
                .to_string(),
        );
    }

    fs::create_dir_all(worktrees_dir()).map_err(|e| e.to_string())?;

    // Capture initial state BEFORE creating worktree
    let initial_untracked = get_untracked_files(&cwd);
    let initial_modified = get_modified_files(&cwd);

    // Capture the exact current state including uncommitted changes using git stash create.
    // This creates a commit object representing the current state without actually stashing.
    let stash_output = Command::new("git")
        .args(["stash", "create"])
        .current_dir(&cwd)
        .output()
        .map_err(|e| format!("git stash create failed: {}", e))?;

    let stash_sha = String::from_utf8_lossy(&stash_output.stdout)
        .trim()
        .to_string();

    // If stash_sha is empty, there are no uncommitted changes; use HEAD
    let base_sha = if stash_sha.is_empty() {
        let head_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&cwd)
            .output()
            .map_err(|e| format!("git rev-parse HEAD failed: {}", e))?;

        if !head_output.status.success() {
            return Err("Not a git repository or no commits yet".to_string());
        }

        String::from_utf8_lossy(&head_output.stdout)
            .trim()
            .to_string()
    } else {
        stash_sha
    };

    // Create the worktree detached at the base_sha (which includes uncommitted changes if any)
    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "--detach",
            &wt_path.to_string_lossy(),
            &base_sha,
        ])
        .current_dir(&cwd)
        .output()
        .map_err(|e| format!("git worktree add failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree add failed: {}", stderr));
    }

    // Copy untracked files from main to worktree so the agent has a complete picture
    copy_untracked_files(&cwd, &wt_path.to_string_lossy())?;

    // Also copy essential ignored files if .worktreeinclude exists
    copy_worktreeinclude_files(&cwd, &wt_path.to_string_lossy())?;

    let now = chrono::Utc::now().to_rfc3339();
    let info = WorktreeInfo {
        session_id: session_id.clone(),
        base_sha,
        worktree_path: wt_path.to_string_lossy().to_string(),
        main_cwd: cwd,
        created_at: now,
        initial_untracked,
        initial_modified,
        provider_id,
        provider_session_id: None,
        claude_session_id: None,
    };
    save_worktree_meta(&info)?;

    Ok(info)
}

/// Copy untracked/ignored files that aren't in git but are needed for builds
fn copy_untracked_files(main_cwd: &str, wt_path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(main_cwd)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let src = Path::new(main_cwd).join(line);
            let dst = Path::new(wt_path).join(line);
            if src.exists() && src.is_file() {
                if let Some(parent) = dst.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::copy(&src, &dst);
            }
        }
    }

    Ok(())
}

/// Copy files listed in .worktreeinclude (essential ignored files like .env, credentials)
fn copy_worktreeinclude_files(main_cwd: &str, wt_path: &str) -> Result<(), String> {
    // Check for .worktreeinclude or .tyck/worktreeinclude
    let include_paths = [
        Path::new(main_cwd).join(".worktreeinclude"),
        Path::new(main_cwd).join(".tyck").join("worktreeinclude"),
    ];

    let mut patterns: Vec<String> = Vec::new();

    for include_path in &include_paths {
        if include_path.exists() {
            if let Ok(content) = fs::read_to_string(include_path) {
                for line in content.lines() {
                    let line = line.trim();
                    // Skip comments and empty lines
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    patterns.push(line.to_string());
                }
            }
        }
    }

    // If no .worktreeinclude, use sensible defaults for common essential files
    if patterns.is_empty() {
        patterns = vec![
            ".env".to_string(),
            ".env.local".to_string(),
            ".env.development".to_string(),
            ".env.development.local".to_string(),
            "config/master.key".to_string(),
            "config/credentials.yml.enc".to_string(),
        ];
    }

    for pattern in &patterns {
        let src = Path::new(main_cwd).join(pattern);
        let dst = Path::new(wt_path).join(pattern);

        if src.exists() && src.is_file() {
            if let Some(parent) = dst.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(&src, &dst);
        }
    }

    Ok(())
}

/// Scan changes in the worktree compared to the base SHA.
/// Excludes files that existed in the main repo when the worktree was created.
#[tauri::command]
pub fn scan_worktree_changes(session_id: String) -> Result<Vec<WorktreeFileDiff>, String> {
    let info = load_worktree_meta(&session_id)?;
    let wt = &info.worktree_path;

    if !Path::new(wt).exists() {
        return Ok(vec![]);
    }

    // Build sets of initial files for quick lookup
    let initial_untracked: HashSet<&str> = info.initial_untracked.iter().map(|s| s.as_str()).collect();
    let initial_modified: HashSet<&str> = info.initial_modified.iter().map(|s| s.as_str()).collect();

    let mut diffs = Vec::new();
    let mut stats: HashMap<String, (i32, i32)> = HashMap::new();

    // numstat: base_sha vs working tree in worktree
    if let Ok(output) = Command::new("git")
        .args(["diff", &info.base_sha, "--numstat"])
        .current_dir(wt)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let add = parts[0].parse::<i32>().unwrap_or(0);
                let del = parts[1].parse::<i32>().unwrap_or(0);
                stats.insert(parts[2].to_string(), (add, del));
            }
        }
    }

    // name-status: get actual changes
    if let Ok(output) = Command::new("git")
        .args(["diff", &info.base_sha, "--name-status"])
        .current_dir(wt)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, |c: char| c == '\t' || c == ' ').collect();
            if parts.len() >= 2 {
                let status = parts[0].trim().to_string();
                let path = parts[1].trim().to_string();

                // Skip files that were already modified when the worktree was created
                // (they're part of the base state, not agent changes)
                if initial_modified.contains(path.as_str()) {
                    // Check if the file content actually differs from initial state
                    // If it's the same, skip it; if different, agent made additional changes
                    if !file_changed_from_initial(&info, &path) {
                        continue;
                    }
                }

                // Skip files that have already been synced to main (accepted)
                // This prevents showing accepted changes when resuming a session
                if file_already_synced_to_main(&info, &path, &status) {
                    continue;
                }

                let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
                diffs.push(WorktreeFileDiff {
                    path,
                    status,
                    additions,
                    deletions,
                });
            }
        }
    }

    // Untracked files in worktree — include if agent created OR modified them
    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(wt)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let current_untracked: HashSet<&str> = text.trim().split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for line in &current_untracked {
            // Skip if already in diffs
            if diffs.iter().any(|d| d.path == *line) {
                continue;
            }
            // If file was untracked at session start, only include if agent modified it
            if initial_untracked.contains(*line) {
                if !untracked_file_changed(&info, line) {
                    continue;
                }
            }
            // Skip files that have already been synced to main (accepted)
            if file_already_synced_to_main(&info, line, "A") {
                continue;
            }
            diffs.push(WorktreeFileDiff {
                path: line.to_string(),
                status: "A".to_string(),
                additions: 0,
                deletions: 0,
            });
        }

        // Check for deleted untracked files (were in initial_untracked but no longer exist in worktree)
        for file in &info.initial_untracked {
            if !current_untracked.contains(file.as_str()) {
                // File was untracked at start but doesn't exist now — agent deleted it
                let wt_file = Path::new(wt).join(file);
                if !wt_file.exists() && !diffs.iter().any(|d| d.path == *file) {
                    // Skip if the deletion has already been synced to main
                    if file_already_synced_to_main(&info, file, "D") {
                        continue;
                    }
                    diffs.push(WorktreeFileDiff {
                        path: file.clone(),
                        status: "D".to_string(),
                        additions: 0,
                        deletions: 0,
                    });
                }
            }
        }
    }

    Ok(diffs)
}

/// Check if a file in the worktree has changed from its initial state in main repo.
/// Used to detect if an agent made additional changes to a file that was
/// already modified when the worktree was created.
fn file_changed_from_initial(info: &WorktreeInfo, file_path: &str) -> bool {
    // Get file content from main repo (the initial state for modified files)
    let main_file = Path::new(&info.main_cwd).join(file_path);
    let main_content = match fs::read(&main_file) {
        Ok(c) => c,
        Err(_) => return true, // If main file doesn't exist, agent changed something
    };

    // Get current worktree content
    let wt_file = Path::new(&info.worktree_path).join(file_path);
    let current_content = match fs::read(&wt_file) {
        Ok(c) => c,
        Err(_) => return true, // If worktree file doesn't exist, it was deleted
    };

    main_content != current_content
}

/// Check if an untracked file in the worktree differs from the main repo.
/// Used to detect if an agent modified an untracked file that existed before the session.
fn untracked_file_changed(info: &WorktreeInfo, file_path: &str) -> bool {
    let wt_file = Path::new(&info.worktree_path).join(file_path);
    let main_file = Path::new(&info.main_cwd).join(file_path);

    let wt_content = match fs::read(&wt_file) {
        Ok(c) => c,
        Err(_) => return false, // Worktree file doesn't exist, no change to show
    };

    let main_content = match fs::read(&main_file) {
        Ok(c) => c,
        Err(_) => return true, // Main file doesn't exist but worktree has it — agent created it
    };

    wt_content != main_content
}

/// Check if a file in the worktree has already been synced to main (accepted).
/// Returns true if the file exists in both locations and content is identical.
fn file_already_synced_to_main(info: &WorktreeInfo, file_path: &str, status: &str) -> bool {
    let wt_file = Path::new(&info.worktree_path).join(file_path);
    let main_file = Path::new(&info.main_cwd).join(file_path);
    
    match status {
        "D" => {
            // Deleted in worktree - check if also deleted in main
            !main_file.exists()
        }
        _ => {
            // Added or Modified - check if content matches main
            let wt_content = match fs::read(&wt_file) {
                Ok(c) => c,
                Err(_) => return false,
            };
            let main_content = match fs::read(&main_file) {
                Ok(c) => c,
                Err(_) => return false,
            };
            wt_content == main_content
        }
    }
}

/// Read a file from the agent's worktree.
#[tauri::command]
pub fn get_file_from_worktree(session_id: String, file_path: String) -> Result<String, String> {
    validate_file_path(&file_path)?;
    let info = load_worktree_meta(&session_id)?;
    let full = Path::new(&info.worktree_path).join(&file_path);
    if full.exists() {
        fs::read_to_string(&full).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

/// Read a file at the base state (before agent started).
/// For files that were untracked at session start, reads from main repo.
/// For tracked files, reads from base_sha in git history.
#[tauri::command]
pub fn get_file_at_base(session_id: String, file_path: String) -> Result<String, String> {
    validate_file_path(&file_path)?;
    let info = load_worktree_meta(&session_id)?;

    // If file was untracked at session start, read from main repo (not git history)
    if info.initial_untracked.contains(&file_path) {
        let main_file = Path::new(&info.main_cwd).join(&file_path);
        return fs::read_to_string(&main_file).or_else(|_| Ok(String::new()));
    }

    // For tracked files, read from base_sha
    let output = Command::new("git")
        .args(["show", &format!("{}:{}", info.base_sha, file_path)])
        .current_dir(&info.main_cwd)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // File didn't exist at base (new file)
        Ok(String::new())
    }
}

/// Accept a single file: copy from worktree to main workspace.
/// Performs three-way conflict detection: if the file in main has diverged
/// from base_sha (i.e. another agent or user edited it), we attempt a merge.
/// If merge conflicts exist, we write conflict markers to the WORKTREE file
/// so the agent can see and resolve them.
#[tauri::command]
pub fn accept_worktree_file(
    session_id: String,
    file_path: String,
    status: String,
    force: Option<bool>,
) -> Result<AcceptResult, String> {
    validate_file_path(&file_path)?;
    let info = load_worktree_meta(&session_id)?;
    let force = force.unwrap_or(false);

    let main_file = Path::new(&info.main_cwd).join(&file_path);
    let wt_file = Path::new(&info.worktree_path).join(&file_path);

    if status == "D" {
        // Agent deleted the file
        if !force {
            // Check if main still has the file in its base state
            if let Some(reason) = check_conflict(&info, &file_path) {
                return Ok(AcceptResult {
                    success: false,
                    conflict: true,
                    conflict_reason: Some(reason),
                    conflict_written: false,
                });
            }
        }
        if main_file.exists() {
            fs::remove_file(&main_file).map_err(|e| e.to_string())?;
            // Clean up empty parent dirs
            if let Some(parent) = main_file.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
        return Ok(AcceptResult {
            success: true,
            conflict: false,
            conflict_reason: None,
            conflict_written: false,
        });
    }

    if !wt_file.exists() {
        return Err(format!("File {} not found in worktree", file_path));
    }

    // For A (added) or M (modified): handle merging
    if !force {
        if status == "M" {
            // For modified files, attempt three-way merge
            match try_three_way_merge(&info, &file_path, &main_file, &wt_file) {
                MergeOutcome::Clean(merged_content) => {
                    // Merge succeeded without conflicts - write merged content to both main AND worktree
                    // Writing to both ensures they stay in sync
                    if let Some(parent) = main_file.parent() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    fs::write(&main_file, &merged_content).map_err(|e| e.to_string())?;
                    fs::write(&wt_file, &merged_content).map_err(|e| e.to_string())?;
                    return Ok(AcceptResult {
                        success: true,
                        conflict: false,
                        conflict_reason: None,
                        conflict_written: false,
                    });
                }
                MergeOutcome::ConflictWithMarkers(merged_content) => {
                    // Write conflict markers to WORKTREE so agent can see/resolve
                    fs::write(&wt_file, &merged_content).map_err(|e| e.to_string())?;
                    return Ok(AcceptResult {
                        success: false,
                        conflict: true,
                        conflict_reason: Some(format!(
                            "File '{}' has conflicting changes. Conflict markers written to worktree.",
                            file_path
                        )),
                        conflict_written: true,
                    });
                }
                MergeOutcome::Error(reason) => {
                    return Ok(AcceptResult {
                        success: false,
                        conflict: true,
                        conflict_reason: Some(reason),
                        conflict_written: false,
                    });
                }
                MergeOutcome::NoMergeNeeded => {
                    // Main hasn't diverged, just copy worktree version
                }
            }
        } else if status == "A" {
            // For added files, check if main workspace has a different version
            // If so, do a two-way merge (no base)
            if main_file.exists() {
                match try_two_way_merge(&main_file, &wt_file, &file_path) {
                    MergeOutcome::Clean(merged_content) => {
                        // Unlikely for two-way with empty base, but handle it
                        // Write to both main AND worktree to keep them in sync
                        if let Some(parent) = main_file.parent() {
                            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                        }
                        fs::write(&main_file, &merged_content).map_err(|e| e.to_string())?;
                        fs::write(&wt_file, &merged_content).map_err(|e| e.to_string())?;
                        return Ok(AcceptResult {
                            success: true,
                            conflict: false,
                            conflict_reason: None,
                            conflict_written: false,
                        });
                    }
                    MergeOutcome::ConflictWithMarkers(merged_content) => {
                        // Write conflict markers to WORKTREE
                        fs::write(&wt_file, &merged_content).map_err(|e| e.to_string())?;
                        return Ok(AcceptResult {
                            success: false,
                            conflict: true,
                            conflict_reason: Some(format!(
                                "File '{}' exists in main with different content. Conflict markers written to worktree.",
                                file_path
                            )),
                            conflict_written: true,
                        });
                    }
                    MergeOutcome::Error(reason) => {
                        return Ok(AcceptResult {
                            success: false,
                            conflict: true,
                            conflict_reason: Some(reason),
                            conflict_written: false,
                        });
                    }
                    MergeOutcome::NoMergeNeeded => {
                        // Files are the same, proceed with copy
                    }
                }
            }
        }
    }

    // Default: copy worktree content to main
    let content = fs::read(&wt_file).map_err(|e| e.to_string())?;
    if let Some(parent) = main_file.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&main_file, content).map_err(|e| e.to_string())?;

    Ok(AcceptResult {
        success: true,
        conflict: false,
        conflict_reason: None,
        conflict_written: false,
    })
}

/// Result of attempting a three-way merge
enum MergeOutcome {
    /// Merge succeeded cleanly - contains the merged content
    Clean(Vec<u8>),
    /// Merge has conflicts - contains merged content WITH conflict markers
    ConflictWithMarkers(Vec<u8>),
    /// Merge failed with error (no content available)
    Error(String),
    /// No merge needed - main hasn't diverged from base
    NoMergeNeeded,
}

/// Attempt a three-way merge for a modified file.
/// Returns Clean(merged_content) if merge succeeds without conflicts,
/// Conflict(reason) if there are conflicts, or NoMergeNeeded if main hasn't changed.
fn try_three_way_merge(
    info: &WorktreeInfo,
    file_path: &str,
    main_file: &Path,
    wt_file: &Path,
) -> MergeOutcome {
    // For initially untracked files, we can't do three-way merge
    if info.initial_untracked.contains(&file_path.to_string()) {
        return MergeOutcome::NoMergeNeeded;
    }

    // Get base content from git
    let base_output = Command::new("git")
        .args(["show", &format!("{}:{}", info.base_sha, file_path)])
        .current_dir(&info.main_cwd)
        .output()
        .ok();

    let base_content = match base_output {
        Some(ref o) if o.status.success() => o.stdout.clone(),
        _ => return MergeOutcome::NoMergeNeeded, // Can't get base, skip merge
    };

    // Get current main workspace content
    let main_content = match fs::read(main_file) {
        Ok(c) => c,
        Err(_) => return MergeOutcome::NoMergeNeeded, // Main doesn't exist or can't read
    };

    // Get worktree content
    let wt_content = match fs::read(wt_file) {
        Ok(c) => c,
        Err(_) => return MergeOutcome::Error("Cannot read worktree file".to_string()),
    };

    // If main already has conflict markers but worktree is clean,
    // the worktree version is the resolved version - use it directly
    if has_conflict_markers(&main_content) && !has_conflict_markers(&wt_content) {
        return MergeOutcome::NoMergeNeeded;
    }

    // If worktree has conflict markers, they need to be resolved first
    if has_conflict_markers(&wt_content) {
        return MergeOutcome::Error(format!(
            "File '{}' in worktree has unresolved conflict markers. Please resolve them first.",
            file_path
        ));
    }

    // If main hasn't changed from base, no merge needed
    if base_content == main_content {
        return MergeOutcome::NoMergeNeeded;
    }

    // If worktree hasn't changed from base, just use main's version (user's edits win)
    if base_content == wt_content {
        return MergeOutcome::Clean(main_content);
    }

    // If main and worktree are the same, no conflict
    if main_content == wt_content {
        return MergeOutcome::NoMergeNeeded;
    }

    // Both diverged - need to attempt merge using git merge-file
    // Create temp files for the merge with unique names to prevent race conditions
    let temp_dir = std::env::temp_dir();
    let prefix = unique_temp_prefix();
    let base_tmp = temp_dir.join(format!("{}_base", prefix));
    let main_tmp = temp_dir.join(format!("{}_main", prefix));
    let wt_tmp = temp_dir.join(format!("{}_wt", prefix));

    if fs::write(&base_tmp, &base_content).is_err()
        || fs::write(&main_tmp, &main_content).is_err()
        || fs::write(&wt_tmp, &wt_content).is_err()
    {
        // Cleanup
        let _ = fs::remove_file(&base_tmp);
        let _ = fs::remove_file(&main_tmp);
        let _ = fs::remove_file(&wt_tmp);
        return MergeOutcome::Error("Failed to create temp files for merge".to_string());
    }

    // git merge-file modifies the first file in-place and returns:
    // 0 = clean merge, >0 = conflicts (number of conflicts), <0 = error
    let merge_result = Command::new("git")
        .args([
            "merge-file",
            "-p",           // Print to stdout instead of modifying file
            "--diff3",      // Show base version in conflicts for clarity
            "-L", "main (workspace)",
            "-L", "base",
            "-L", "agent (worktree)",
            main_tmp.to_str().unwrap_or(""),
            base_tmp.to_str().unwrap_or(""),
            wt_tmp.to_str().unwrap_or(""),
        ])
        .current_dir(&info.main_cwd)
        .output();

    // Cleanup temp files
    let _ = fs::remove_file(&base_tmp);
    let _ = fs::remove_file(&main_tmp);
    let _ = fs::remove_file(&wt_tmp);

    match merge_result {
        Ok(output) => {
            if output.status.success() {
                // Exit code 0 = clean merge
                MergeOutcome::Clean(output.stdout)
            } else if output.status.code() == Some(1) {
                // Exit code 1 = conflicts exist, output.stdout contains merged content with markers
                MergeOutcome::ConflictWithMarkers(output.stdout)
            } else {
                // Other exit codes = error
                MergeOutcome::Error(format!(
                    "Merge failed for '{}': {}",
                    file_path,
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
        Err(e) => MergeOutcome::Error(format!("Failed to run git merge-file: {}", e)),
    }
}

/// Two-way merge for untracked files (no base available).
/// Produces conflict markers showing main vs worktree versions.
/// Check if content contains git conflict markers
fn has_conflict_markers(content: &[u8]) -> bool {
    let text = String::from_utf8_lossy(content);
    text.contains("<<<<<<<") && text.contains("=======") && text.contains(">>>>>>>")
}

fn try_two_way_merge(
    main_file: &Path,
    wt_file: &Path,
    file_path: &str,
) -> MergeOutcome {
    // Read both versions
    let main_content = match fs::read(main_file) {
        Ok(c) => c,
        Err(_) => return MergeOutcome::NoMergeNeeded, // Main doesn't exist
    };

    let wt_content = match fs::read(wt_file) {
        Ok(c) => c,
        Err(_) => return MergeOutcome::Error("Cannot read worktree file".to_string()),
    };

    // If they're the same, no conflict
    if main_content == wt_content {
        return MergeOutcome::NoMergeNeeded;
    }

    // If main already has conflict markers but worktree is clean,
    // the worktree version is the resolved version - use it directly
    if has_conflict_markers(&main_content) && !has_conflict_markers(&wt_content) {
        return MergeOutcome::NoMergeNeeded;
    }

    // If worktree has conflict markers, they need to be resolved first
    if has_conflict_markers(&wt_content) {
        return MergeOutcome::Error(format!(
            "File '{}' in worktree has unresolved conflict markers. Please resolve them first.",
            file_path
        ));
    }

    // Use empty base for two-way merge (this will show both versions as conflicting)
    let temp_dir = std::env::temp_dir();
    let prefix = unique_temp_prefix();
    let base_tmp = temp_dir.join(format!("{}_base", prefix));
    let main_tmp = temp_dir.join(format!("{}_main", prefix));
    let wt_tmp = temp_dir.join(format!("{}_wt", prefix));

    // Empty base means both sides diverged from "nothing"
    if fs::write(&base_tmp, b"").is_err()
        || fs::write(&main_tmp, &main_content).is_err()
        || fs::write(&wt_tmp, &wt_content).is_err()
    {
        let _ = fs::remove_file(&base_tmp);
        let _ = fs::remove_file(&main_tmp);
        let _ = fs::remove_file(&wt_tmp);
        return MergeOutcome::Error("Failed to create temp files for merge".to_string());
    }

    let merge_result = Command::new("git")
        .args([
            "merge-file",
            "-p",
            "-L", "main (workspace)",
            "-L", "base",
            "-L", "agent (worktree)",
            main_tmp.to_str().unwrap_or(""),
            base_tmp.to_str().unwrap_or(""),
            wt_tmp.to_str().unwrap_or(""),
        ])
        .output();

    let _ = fs::remove_file(&base_tmp);
    let _ = fs::remove_file(&main_tmp);
    let _ = fs::remove_file(&wt_tmp);

    match merge_result {
        Ok(output) => {
            if output.status.success() {
                MergeOutcome::Clean(output.stdout)
            } else if output.status.code() == Some(1) {
                // Conflicts - return content with markers
                MergeOutcome::ConflictWithMarkers(output.stdout)
            } else {
                MergeOutcome::Error(format!(
                    "Two-way merge failed for '{}': {}",
                    file_path,
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
        Err(e) => MergeOutcome::Error(format!("Failed to run git merge-file: {}", e)),
    }
}

/// Three-way conflict detection: check if the file in the main workspace
/// has diverged from what it was when the session started.
/// For initially untracked files, we stored their state in initial_untracked,
/// so conflict detection compares worktree vs main (they should match if no external edits).
fn check_conflict(info: &WorktreeInfo, file_path: &str) -> Option<String> {
    let main_file = Path::new(&info.main_cwd).join(file_path);

    // For files that were untracked at session start, we can't use git history.
    // Instead, we need to detect if main was modified externally by checking if
    // the main content differs from what we expect (i.e., worktree started from main).
    // However, the agent may have modified the worktree, so we just check if main exists
    // and was modified since session start (we don't have a baseline for untracked files).
    // For now, skip conflict detection for initially untracked files.
    if info.initial_untracked.contains(&file_path.to_string()) {
        // For untracked files, no reliable conflict detection — allow overwrite
        return None;
    }

    // Get file content at base_sha (for tracked files)
    let base_output = Command::new("git")
        .args(["show", &format!("{}:{}", info.base_sha, file_path)])
        .current_dir(&info.main_cwd)
        .output()
        .ok();

    let base_content = base_output
        .as_ref()
        .filter(|o| o.status.success())
        .map(|o| o.stdout.clone());

    // Get current main workspace content
    let main_content = fs::read(&main_file).ok();

    match (base_content, main_content) {
        (Some(base), Some(main)) => {
            if base != main {
                Some(format!(
                    "File '{}' was modified in the main workspace since the agent started. \
                     The main version differs from the base.",
                    file_path
                ))
            } else {
                None
            }
        }
        (None, Some(_)) => {
            // File didn't exist at base but exists in main now — someone else created it
            Some(format!(
                "File '{}' was created in the main workspace by another process.",
                file_path
            ))
        }
        (Some(_), None) => {
            // File existed at base but was deleted in main
            Some(format!(
                "File '{}' was deleted from the main workspace since the agent started.",
                file_path
            ))
        }
        (None, None) => None, // Neither existed — no conflict (new file from agent)
    }
}

/// Three-way merge result
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    /// Merged content (with conflict markers if has_conflicts)
    pub content: String,
    /// Whether the merge has unresolved conflicts
    pub has_conflicts: bool,
    /// Base content (before either party changed it)
    pub base_content: String,
    /// "Yours" = main workspace version
    pub yours_content: String,
    /// "Theirs" = agent worktree version
    pub theirs_content: String,
}

/// Execute git merge-file with the given contents.
/// Returns (merged_content, has_conflicts).
fn execute_git_merge(
    base_content: &[u8],
    main_content: &[u8],
    wt_content: &[u8],
) -> Result<(Vec<u8>, bool), String> {
    let temp_dir = std::env::temp_dir();
    let prefix = unique_temp_prefix();
    let base_tmp = temp_dir.join(format!("{}_base", prefix));
    let main_tmp = temp_dir.join(format!("{}_main", prefix));
    let wt_tmp = temp_dir.join(format!("{}_wt", prefix));

    fs::write(&base_tmp, base_content).map_err(|e| e.to_string())?;
    fs::write(&main_tmp, main_content).map_err(|e| e.to_string())?;
    fs::write(&wt_tmp, wt_content).map_err(|e| e.to_string())?;

    let output = Command::new("git")
        .args([
            "merge-file",
            "-p",
            "--diff3",
            "-L", "main (workspace)",
            "-L", "base",
            "-L", "agent (worktree)",
            main_tmp.to_str().unwrap_or(""),
            base_tmp.to_str().unwrap_or(""),
            wt_tmp.to_str().unwrap_or(""),
        ])
        .output();

    // Clean up temp files
    let _ = fs::remove_file(&base_tmp);
    let _ = fs::remove_file(&main_tmp);
    let _ = fs::remove_file(&wt_tmp);

    match output {
        Ok(o) => {
            let has_conflicts = !o.status.success() && o.status.code() == Some(1);
            Ok((o.stdout, has_conflicts))
        }
        Err(e) => Err(format!("Failed to run git merge-file: {}", e)),
    }
}

/// Perform a three-way merge for a conflicted file.
/// Returns all three versions plus the merged result (with conflict markers if needed).
#[tauri::command]
pub fn three_way_merge(session_id: String, file_path: String) -> Result<MergeResult, String> {
    validate_file_path(&file_path)?;
    let info = load_worktree_meta(&session_id)?;

    // 1. Base content - for initially untracked files, use empty string (no history)
    let base_content = if info.initial_untracked.contains(&file_path) {
        String::new()
    } else {
        let base_output = Command::new("git")
            .args(["show", &format!("{}:{}", info.base_sha, file_path)])
            .current_dir(&info.main_cwd)
            .output()
            .map_err(|e| e.to_string())?;
        if base_output.status.success() {
            String::from_utf8_lossy(&base_output.stdout).to_string()
        } else {
            String::new()
        }
    };

    // 2. "Yours" = current main workspace
    let main_file = Path::new(&info.main_cwd).join(&file_path);
    let yours_content = if main_file.exists() {
        fs::read_to_string(&main_file).unwrap_or_default()
    } else {
        String::new()
    };

    // 3. "Theirs" = agent worktree version
    let wt_file = Path::new(&info.worktree_path).join(&file_path);
    let theirs_content = if wt_file.exists() {
        fs::read_to_string(&wt_file).unwrap_or_default()
    } else {
        String::new()
    };

    // 4. Execute merge using shared helper
    let (merged_bytes, has_conflicts) = execute_git_merge(
        base_content.as_bytes(),
        yours_content.as_bytes(),
        theirs_content.as_bytes(),
    )?;

    let content = String::from_utf8_lossy(&merged_bytes).to_string();

    Ok(MergeResult {
        content,
        has_conflicts,
        base_content,
        yours_content,
        theirs_content,
    })
}

/// Write a resolved merge result to both main workspace and worktree.
/// Writing to both ensures they're in sync so the file won't reappear as pending.
#[tauri::command]
pub fn resolve_conflict(
    session_id: String,
    file_path: String,
    resolved_content: String,
) -> Result<(), String> {
    validate_file_path(&file_path)?;
    let info = load_worktree_meta(&session_id)?;
    
    // Write to main workspace
    let main_file = Path::new(&info.main_cwd).join(&file_path);
    if let Some(parent) = main_file.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&main_file, &resolved_content).map_err(|e| e.to_string())?;
    
    // Also write to worktree to keep them in sync
    let wt_file = Path::new(&info.worktree_path).join(&file_path);
    if let Some(parent) = wt_file.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&wt_file, &resolved_content).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Finalize a session review: save record and optionally clean up worktree.
#[tauri::command]
pub fn finalize_session_review(
    session_id: String,
    decisions: HashMap<String, String>,
    cleanup: Option<bool>,
) -> Result<(), String> {
    let info = load_worktree_meta(&session_id)?;
    let should_cleanup = cleanup.unwrap_or(true);

    // Scan current diffs to get statuses
    let diffs = scan_worktree_changes(session_id.clone())?;

    let mut reviewed_files = Vec::new();
    for diff in &diffs {
        let decision = decisions
            .get(&diff.path)
            .map(|s| s.as_str())
            .unwrap_or("rejected");
        reviewed_files.push(ReviewedFileRecord {
            path: diff.path.clone(),
            status: diff.status.clone(),
            decision: decision.to_string(),
        });
    }

    // Save review record
    let record = SessionReviewRecord {
        session_id: session_id.clone(),
        main_cwd: info.main_cwd.clone(),
        created_at: info.created_at.clone(),
        completed_at: chrono::Utc::now().to_rfc3339(),
        files: reviewed_files,
    };
    let reviews = reviews_dir();
    fs::create_dir_all(&reviews).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(reviews.join(format!("{}.json", session_id)), json).map_err(|e| e.to_string())?;

    if should_cleanup {
        cleanup_worktree_inner(&info)?;
    }

    Ok(())
}

/// Remove a worktree and its metadata.
#[tauri::command]
pub fn cleanup_worktree(session_id: String) -> Result<(), String> {
    let info = load_worktree_meta(&session_id)?;
    cleanup_worktree_inner(&info)
}

fn cleanup_worktree_inner(info: &WorktreeInfo) -> Result<(), String> {
    let wt = Path::new(&info.worktree_path);

    // First, unlock the worktree in case it's locked (safe even if not locked)
    let _ = Command::new("git")
        .args(["worktree", "unlock", &info.worktree_path])
        .current_dir(&info.main_cwd)
        .output();

    if wt.exists() {
        // Use git worktree remove for clean removal
        let output = Command::new("git")
            .args(["worktree", "remove", "--force", &info.worktree_path])
            .current_dir(&info.main_cwd)
            .output();

        match output {
            Ok(o) if o.status.success() => {}
            _ => {
                // Fallback: manually remove directory and prune
                let _ = fs::remove_dir_all(wt);
                let _ = Command::new("git")
                    .args(["worktree", "prune"])
                    .current_dir(&info.main_cwd)
                    .output();
            }
        }
    } else {
        // Worktree dir doesn't exist but git might still have a reference
        let _ = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&info.main_cwd)
            .output();
    }

    // NOTE: We intentionally keep the metadata file so that session discovery
    // can still find sessions that ran in this worktree. Claude stores its
    // session files keyed by the worktree path, so we need the metadata to
    // map from main_cwd to worktree_path for collect_valid_cwds().
    // The cleanup_stale_worktrees() function will eventually remove truly
    // stale metadata files.

    Ok(())
}

/// Get the worktree path for a session (used to pass as cwd to agent).
#[tauri::command]
pub fn get_worktree_path(session_id: String) -> Result<String, String> {
    let info = load_worktree_meta(&session_id)?;
    Ok(info.worktree_path)
}

/// Check if a worktree has pending (unsynced) changes.
/// Returns true if there are files in the worktree that differ from main workspace.
#[tauri::command]
pub fn worktree_has_pending_changes(session_id: String) -> Result<bool, String> {
    match scan_worktree_changes(session_id) {
        Ok(diffs) => Ok(!diffs.is_empty()),
        Err(_) => Ok(false), // No worktree or error - no pending changes
    }
}

/// Debug: check why a specific file is showing as pending.
/// Returns details about the file comparison.
#[tauri::command]
pub fn debug_file_sync_status(session_id: String, file_path: String) -> Result<String, String> {
    let info = load_worktree_meta(&session_id)?;
    let wt_file = Path::new(&info.worktree_path).join(&file_path);
    let main_file = Path::new(&info.main_cwd).join(&file_path);
    
    let mut result = String::new();
    result.push_str(&format!("Worktree: {}\n", info.worktree_path));
    result.push_str(&format!("Main CWD: {}\n", info.main_cwd));
    result.push_str(&format!("File: {}\n\n", file_path));
    
    result.push_str(&format!("Worktree file exists: {}\n", wt_file.exists()));
    result.push_str(&format!("Main file exists: {}\n", main_file.exists()));
    
    if wt_file.exists() {
        if let Ok(meta) = fs::metadata(&wt_file) {
            result.push_str(&format!("Worktree file size: {} bytes\n", meta.len()));
        }
    }
    
    if main_file.exists() {
        if let Ok(meta) = fs::metadata(&main_file) {
            result.push_str(&format!("Main file size: {} bytes\n", meta.len()));
        }
    }
    
    // Compare content
    let wt_content = fs::read(&wt_file).ok();
    let main_content = fs::read(&main_file).ok();
    
    match (&wt_content, &main_content) {
        (Some(wt), Some(main)) => {
            if wt == main {
                result.push_str("\nContent: IDENTICAL\n");
            } else {
                result.push_str(&format!("\nContent: DIFFERENT\n"));
                result.push_str(&format!("  Worktree bytes: {}\n", wt.len()));
                result.push_str(&format!("  Main bytes: {}\n", main.len()));
                
                // Check for line ending differences
                let wt_str = String::from_utf8_lossy(wt);
                let main_str = String::from_utf8_lossy(main);
                let wt_crlf = wt_str.matches("\r\n").count();
                let main_crlf = main_str.matches("\r\n").count();
                let wt_lf = wt_str.matches('\n').count() - wt_crlf;
                let main_lf = main_str.matches('\n').count() - main_crlf;
                
                result.push_str(&format!("  Worktree: {} LF, {} CRLF\n", wt_lf, wt_crlf));
                result.push_str(&format!("  Main: {} LF, {} CRLF\n", main_lf, main_crlf));
                
                // Show first difference
                for (i, (a, b)) in wt.iter().zip(main.iter()).enumerate() {
                    if a != b {
                        result.push_str(&format!("  First diff at byte {}: wt={} main={}\n", i, a, b));
                        break;
                    }
                }
                if wt.len() != main.len() {
                    result.push_str(&format!("  Length diff: wt has {} more bytes\n", 
                        wt.len() as i64 - main.len() as i64));
                }
            }
        }
        (None, Some(_)) => result.push_str("\nWorktree file could not be read\n"),
        (Some(_), None) => result.push_str("\nMain file could not be read\n"),
        (None, None) => result.push_str("\nNeither file could be read\n"),
    }
    
    // Check initial_untracked
    if info.initial_untracked.contains(&file_path) {
        result.push_str(&format!("\nFile was in initial_untracked\n"));
    }
    if info.initial_modified.contains(&file_path) {
        result.push_str(&format!("File was in initial_modified\n"));
    }
    
    Ok(result)
}

/// Start background discovery of provider session ID for a worktree.
/// Polls every 5 seconds until found or stopped.
#[tauri::command]
pub fn start_provider_session_discovery(worktree_session_id: String) -> Result<(), String> {
    let stop_flag = std::sync::Arc::new(AtomicBool::new(false));
    
    {
        let mut tasks = get_discovery_tasks();
        let tasks_map = tasks.as_mut().unwrap();
        
        // If already running, don't start another
        if tasks_map.contains_key(&worktree_session_id) {
            return Ok(());
        }
        
        tasks_map.insert(worktree_session_id.clone(), stop_flag.clone());
    }
    
    let session_id = worktree_session_id.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            
            // Check if we should stop
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            
            // Try to discover the session
            match discover_provider_session_for_worktree(session_id.clone()) {
                Ok(Some(_)) => {
                    // Found it, stop polling
                    break;
                }
                Ok(None) => {
                    // Not found yet, continue polling
                }
                Err(_) => {
                    // Error (worktree might be gone), stop polling
                    break;
                }
            }
        }
        
        // Clean up
        let mut tasks = get_discovery_tasks();
        if let Some(tasks_map) = tasks.as_mut() {
            tasks_map.remove(&session_id);
        }
    });
    
    Ok(())
}

/// Stop background discovery for a worktree session.
#[tauri::command]
pub fn stop_provider_session_discovery(worktree_session_id: String) -> Result<(), String> {
    let mut tasks = get_discovery_tasks();
    if let Some(tasks_map) = tasks.as_mut() {
        if let Some(stop_flag) = tasks_map.remove(&worktree_session_id) {
            stop_flag.store(true, Ordering::Relaxed);
        }
    }
    Ok(())
}

/// Associate a provider session ID with a worktree.
/// This is called after the agent starts to record which provider session is using this worktree.
#[tauri::command]
pub fn set_worktree_provider_session(
    worktree_session_id: String,
    provider_session_id: String,
) -> Result<(), String> {
    let mut info = load_worktree_meta(&worktree_session_id)?;
    info.set_provider_session_id(provider_session_id);
    save_worktree_meta(&info)
}

/// Find a worktree by its associated provider session ID.
/// Returns the worktree session ID if found.
#[tauri::command]
pub fn find_worktree_by_provider_session(provider_session_id: String) -> Result<Option<String>, String> {
    let dir = worktrees_dir();
    if !dir.exists() {
        return Ok(None);
    }

    let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(info) = serde_json::from_str::<WorktreeInfo>(&data) {
                if info.get_provider_session_id() == Some(&provider_session_id) {
                    // Also check if worktree directory still exists
                    if Path::new(&info.worktree_path).exists() {
                        return Ok(Some(info.session_id));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Find a worktree for resume.
/// 
/// Strategy 1: Look up by provider_session_id in worktree metadata
/// Strategy 2: Find any worktree for this main_cwd that has pending changes (fallback)
/// 
/// Returns (worktree_session_id, worktree_path) if found.
#[tauri::command]
pub fn find_worktree_for_resume(
    provider_session_id: String,
    main_cwd: String,
) -> Result<Option<(String, String)>, String> {
    let dir = worktrees_dir();
    if !dir.exists() {
        return Ok(None);
    }

    // Strategy 1: Direct lookup by provider_session_id
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(info) = serde_json::from_str::<WorktreeInfo>(&data) {
                    if info.get_provider_session_id() == Some(&provider_session_id) {
                        if Path::new(&info.worktree_path).exists() {
                            return Ok(Some((info.session_id, info.worktree_path)));
                        }
                    }
                }
            }
        }
    }

    // Strategy 2: Find any worktree for this main_cwd that has pending changes
    // This is a fallback for when the provider_session_id association was never made
    if let Ok(entries) = fs::read_dir(&dir) {
        let mut candidates: Vec<WorktreeInfo> = Vec::new();
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(info) = serde_json::from_str::<WorktreeInfo>(&data) {
                    // Must match main_cwd and worktree must exist
                    if info.main_cwd == main_cwd && Path::new(&info.worktree_path).exists() {
                        // Check if this worktree has pending changes
                        if let Ok(diffs) = scan_worktree_changes(info.session_id.clone()) {
                            if !diffs.is_empty() {
                                candidates.push(info);
                            }
                        }
                    }
                }
            }
        }

        // If multiple candidates, prefer the most recently created one
        if !candidates.is_empty() {
            candidates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            let best = &candidates[0];
            return Ok(Some((best.session_id.clone(), best.worktree_path.clone())));
        }
    }

    Ok(None)
}

/// Discover provider session ID for a worktree by scanning provider-specific directories.
/// This is called after the agent starts to find which session the provider created.
/// Dispatches to the appropriate provider-specific discovery logic.
#[tauri::command]
pub fn discover_provider_session_for_worktree(worktree_session_id: String) -> Result<Option<String>, String> {
    let info = load_worktree_meta(&worktree_session_id)?;
    
    let provider_id = match info.provider_id.as_deref() {
        Some(id) if !id.is_empty() => id,
        _ => return Ok(None),
    };

    let session_id = match provider_id {
        "claude-code" => discover_claude_session(&info)?,
        "codex" => discover_codex_session(&info)?,
        _ => None,
    };
    
    if let Some(ref sid) = session_id {
        let mut updated_info = info;
        updated_info.set_provider_session_id(sid.clone());
        save_worktree_meta(&updated_info)?;
    }
    
    Ok(session_id)
}

/// Claude-specific session discovery.
/// Scans ~/.claude/projects/<hash>/*.jsonl for sessions matching the worktree path.
fn discover_claude_session(info: &WorktreeInfo) -> Result<Option<String>, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let projects_dir = PathBuf::from(&home).join(".claude").join("projects");
    
    // Convert worktree path to Claude's hash format
    let wt_hash = info.worktree_path
        .replace('/', "-")
        .replace(' ', "-")
        .replace('.', "-");
    
    let project_dir = projects_dir.join(&wt_hash);
    if !project_dir.exists() {
        return Ok(None);
    }
    
    // Find session files and match by cwd
    if let Ok(entries) = fs::read_dir(&project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            
            // Read first few lines to check cwd
            if let Ok(file) = fs::File::open(&path) {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(file);
                
                for line in reader.lines().take(10) {
                    if let Ok(line) = line {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                            // Check if cwd matches our worktree path
                            if let Some(cwd) = val.get("cwd").and_then(|c| c.as_str()) {
                                if cwd == info.worktree_path {
                                    let session_id = path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("")
                                        .to_string();
                                    
                                    if !session_id.is_empty() {
                                        return Ok(Some(session_id));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Codex-specific session discovery.
/// Scans ~/.codex/sessions for rollout files matching the worktree path.
fn discover_codex_session(info: &WorktreeInfo) -> Result<Option<String>, String> {
    let codex_home = std::env::var("CODEX_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.codex", home)
        });
    let sessions_dir = PathBuf::from(&codex_home).join("sessions");
    
    if !sessions_dir.exists() {
        return Ok(None);
    }
    
    // Codex stores sessions at ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
    // We need to recursively scan and check session_meta.cwd
    discover_codex_session_recursive(&sessions_dir, &info.worktree_path)
}

fn discover_codex_session_recursive(dir: &PathBuf, worktree_path: &str) -> Result<Option<String>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(Some(id)) = discover_codex_session_recursive(&path, worktree_path) {
                return Ok(Some(id));
            }
            continue;
        }
        
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if !name.starts_with("rollout-") || !name.ends_with(".jsonl") {
            continue;
        }
        
        // Check first line for session_meta with matching cwd
        if let Ok(file) = fs::File::open(&path) {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(file);
            if let Some(Ok(line)) = reader.lines().next() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                    if val["type"].as_str() == Some("session_meta") {
                        let payload = &val["payload"];
                        if payload["cwd"].as_str() == Some(worktree_path) {
                            if let Some(id) = payload["id"].as_str() {
                                return Ok(Some(id.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Clean up orphaned worktrees on app startup.
/// This handles:
/// - Metadata files for worktrees that no longer exist
/// - Worktrees whose main repo no longer exists
/// - Stale worktrees older than STALE_WORKTREE_DAYS
#[tauri::command]
pub fn cleanup_stale_worktrees() -> Result<u32, String> {
    let dir = worktrees_dir();
    if !dir.exists() {
        return Ok(0);
    }

    let mut cleaned = 0u32;
    let mut main_repos: HashSet<String> = HashSet::new();
    let mut to_clean: Vec<WorktreeInfo> = Vec::new();

    // First pass: collect all worktree info and identify stale ones
    let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        // Only process .json meta files
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(info) = serde_json::from_str::<WorktreeInfo>(&data) {
                    main_repos.insert(info.main_cwd.clone());

                    let wt = Path::new(&info.worktree_path);
                    let main_exists = Path::new(&info.main_cwd).exists();
                    let wt_exists = wt.exists();

                    // Check if worktree is stale (older than threshold)
                    let is_stale = if let Ok(created) =
                        chrono::DateTime::parse_from_rfc3339(&info.created_at)
                    {
                        let age = chrono::Utc::now().signed_duration_since(created);
                        age.num_days() > STALE_WORKTREE_DAYS
                    } else {
                        false
                    };

                    // Clean up if: worktree missing, main repo missing, or stale
                    if !wt_exists || !main_exists || is_stale {
                        to_clean.push(info);
                    }
                }
            }
        }
    }

    // Second pass: run git worktree prune on all known main repos
    for repo in &main_repos {
        if Path::new(repo).exists() && is_git_repo(repo) {
            let _ = Command::new("git")
                .args(["worktree", "prune"])
                .current_dir(repo)
                .output();
        }
    }

    // Third pass: clean up identified stale worktrees
    for info in &to_clean {
        // Try to clean up properly
        if Path::new(&info.main_cwd).exists() && is_git_repo(&info.main_cwd) {
            // Unlock first
            let _ = Command::new("git")
                .args(["worktree", "unlock", &info.worktree_path])
                .current_dir(&info.main_cwd)
                .output();

            // Remove worktree
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", &info.worktree_path])
                .current_dir(&info.main_cwd)
                .output();
        }

        // Remove worktree directory if it still exists
        let wt = Path::new(&info.worktree_path);
        if wt.exists() {
            let _ = fs::remove_dir_all(wt);
        }

        // Remove metadata
        let _ = fs::remove_file(worktree_meta_path(&info.session_id));
        cleaned += 1;
    }

    Ok(cleaned)
}
