use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointInfo {
    pub checkpoint_id: String,
    pub sha: String,       // pre-agent state
    #[serde(default)]
    pub agent_sha: String, // post-agent state (set after stash)
    pub created_at: String,
    pub is_git: bool,
    pub cwd: String,
    #[serde(default)]
    pub stashed: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub status: String, // A, M, D
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRecord {
    pub checkpoint_id: String,
    pub cwd: String,
    pub created_at: String,
    pub completed_at: String,
    pub files: Vec<ReviewedFile>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReviewedFile {
    pub path: String,
    pub status: String,
    pub decision: String,
}

fn tyck_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".tyck")
}

fn checkpoints_dir() -> PathBuf {
    tyck_home().join("checkpoints")
}

fn reviews_dir() -> PathBuf {
    tyck_home().join("reviews")
}

fn checkpoint_meta_path(checkpoint_id: &str) -> PathBuf {
    checkpoints_dir().join(format!("{}.json", checkpoint_id))
}

fn checkpoint_files_dir(checkpoint_id: &str) -> PathBuf {
    checkpoints_dir().join(checkpoint_id)
}

fn is_git_repo(cwd: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_repo_root(cwd: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("Not a git repository".to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn save_checkpoint_meta(info: &CheckpointInfo) -> Result<(), String> {
    let dir = checkpoints_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(info).map_err(|e| e.to_string())?;
    fs::write(checkpoint_meta_path(&info.checkpoint_id), json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_checkpoint_meta(checkpoint_id: &str) -> Result<CheckpointInfo, String> {
    let path = checkpoint_meta_path(checkpoint_id);
    let data = fs::read_to_string(&path)
        .map_err(|_| format!("Checkpoint {} not found", checkpoint_id))?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

/// Resolve the SHA to diff against: checkpoint SHA if available, otherwise HEAD
fn resolve_diff_sha(cwd: &str, checkpoint_id: &str) -> Option<String> {
    if let Ok(info) = load_checkpoint_meta(checkpoint_id) {
        if !info.sha.is_empty() {
            return Some(info.sha);
        }
    }
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }
    None
}

/// Parse git diff between two refs into FileDiff structs
fn parse_git_diffs_between(cwd: &str, from_sha: &str, to_sha: &str) -> Vec<FileDiff> {
    let mut diffs = Vec::new();
    let mut stats: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();

    // numstat
    if let Ok(output) = Command::new("git")
        .args(["diff", "--numstat", from_sha, to_sha])
        .current_dir(cwd)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let add = parts[0].parse::<i32>().unwrap_or(0);
                let del = parts[1].parse::<i32>().unwrap_or(0);
                stats.insert(parts[2].to_string(), (add, del));
            }
        }
    }

    // name-status
    if let Ok(output) = Command::new("git")
        .args(["diff", "--name-status", from_sha, to_sha])
        .current_dir(cwd)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.splitn(2, |c: char| c == '\t' || c == ' ').collect();
            if parts.len() >= 2 {
                let status = parts[0].trim().to_string();
                let path = parts[1].trim().to_string();
                let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
                diffs.push(FileDiff { path, status, additions, deletions });
            }
        }
    }

    diffs
}

/// Parse git diff of working tree vs a ref
fn parse_git_diffs(cwd: &str, sha: &str) -> Vec<FileDiff> {
    let mut diffs = Vec::new();
    let mut stats: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();

    if let Ok(output) = Command::new("git")
        .args(["diff", sha, "--numstat"])
        .current_dir(cwd)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let add = parts[0].parse::<i32>().unwrap_or(0);
                let del = parts[1].parse::<i32>().unwrap_or(0);
                stats.insert(parts[2].to_string(), (add, del));
            }
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["diff", sha, "--name-status"])
        .current_dir(cwd)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.splitn(2, |c: char| c == '\t' || c == ' ').collect();
            if parts.len() >= 2 {
                let status = parts[0].trim().to_string();
                let path = parts[1].trim().to_string();
                let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
                diffs.push(FileDiff { path, status, additions, deletions });
            }
        }
    }

    // Untracked files
    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(cwd)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.trim().split('\n') {
            let line = line.trim();
            if !line.is_empty() && !diffs.iter().any(|d| d.path == line) {
                diffs.push(FileDiff {
                    path: line.to_string(),
                    status: "A".to_string(),
                    additions: 0,
                    deletions: 0,
                });
            }
        }
    }

    diffs
}

// ── Collect files (non-git) ──

fn collect_files(cwd: &str) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_files_recursive(Path::new(cwd), Path::new(cwd), &mut files)?;
    Ok(files)
}

fn collect_files_recursive(base: &Path, dir: &Path, files: &mut Vec<String>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "__pycache__" {
            continue;
        }
        if path.is_dir() {
            collect_files_recursive(base, &path, files)?;
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                files.push(rel.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
}

fn scan_non_git_changes(cwd: &str, snap_dir: &Path) -> Result<Vec<FileDiff>, String> {
    let mut diffs = Vec::new();
    let snap_files = collect_files(&snap_dir.to_string_lossy())?;

    for rel_path in &snap_files {
        let current = Path::new(cwd).join(rel_path);
        let snapshot = snap_dir.join(rel_path);

        if !current.exists() {
            diffs.push(FileDiff {
                path: rel_path.clone(),
                status: "D".to_string(),
                additions: 0,
                deletions: 0,
            });
        } else {
            let snap_content = fs::read_to_string(&snapshot).unwrap_or_default();
            let cur_content = fs::read_to_string(&current).unwrap_or_default();
            if snap_content != cur_content {
                diffs.push(FileDiff {
                    path: rel_path.clone(),
                    status: "M".to_string(),
                    additions: cur_content.lines().count() as i32,
                    deletions: snap_content.lines().count() as i32,
                });
            }
        }
    }

    let cur_files = collect_files(cwd)?;
    for rel_path in &cur_files {
        if !snap_files.contains(rel_path) {
            diffs.push(FileDiff {
                path: rel_path.clone(),
                status: "A".to_string(),
                additions: 0,
                deletions: 0,
            });
        }
    }

    Ok(diffs)
}

// ── Commands ──

#[tauri::command]
pub fn create_checkpoint(cwd: String, session_id: String) -> Result<CheckpointInfo, String> {
    let now = chrono::Utc::now().to_rfc3339();

    if is_git_repo(&cwd) {
        let stash_output = Command::new("git")
            .args(["stash", "create"])
            .current_dir(&cwd)
            .output()
            .map_err(|e| e.to_string())?;

        let sha = String::from_utf8_lossy(&stash_output.stdout).trim().to_string();
        let sha = if sha.is_empty() {
            let head_output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&cwd)
                .output()
                .map_err(|e| e.to_string())?;
            if !head_output.status.success() {
                String::new()
            } else {
                String::from_utf8_lossy(&head_output.stdout).trim().to_string()
            }
        } else {
            sha
        };

        let info = CheckpointInfo {
            checkpoint_id: session_id,
            sha,
            agent_sha: String::new(),
            created_at: now,
            is_git: true,
            cwd: cwd.clone(),
            stashed: false,
        };
        save_checkpoint_meta(&info)?;
        Ok(info)
    } else {
        let dest = checkpoint_files_dir(&session_id);
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;

        let files = collect_files(&cwd)?;
        for rel_path in &files {
            let src = Path::new(&cwd).join(rel_path);
            let dst = dest.join(rel_path);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(&src, &dst).map_err(|e| format!("Copy failed for {}: {}", rel_path, e))?;
        }

        let info = CheckpointInfo {
            checkpoint_id: session_id,
            sha: String::new(),
            agent_sha: String::new(),
            created_at: now,
            is_git: false,
            cwd: cwd.clone(),
            stashed: false,
        };
        save_checkpoint_meta(&info)?;
        Ok(info)
    }
}

/// After the agent finishes: capture agent changes, then restore disk to checkpoint state.
#[tauri::command]
pub fn stash_agent_changes(cwd: String, checkpoint_id: String) -> Result<bool, String> {
    let mut info = load_checkpoint_meta(&checkpoint_id)?;

    if info.is_git {
        // 1. Stage everything (including new untracked files) so stash captures them
        let _ = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&cwd)
            .output();

        // 2. Create stash commit (dangling — doesn't touch stash reflog)
        let stash_output = Command::new("git")
            .args(["stash", "create"])
            .current_dir(&cwd)
            .output()
            .map_err(|e| e.to_string())?;

        let agent_sha = String::from_utf8_lossy(&stash_output.stdout).trim().to_string();
        if agent_sha.is_empty() {
            // No changes — agent didn't modify anything
            // Unstage
            let _ = Command::new("git").args(["reset"]).current_dir(&cwd).output();
            return Ok(false);
        }

        // 3. Find agent-created files (new files not in checkpoint)
        let added_output = Command::new("git")
            .args(["diff", "--name-only", "--diff-filter=A", &info.sha, &agent_sha])
            .current_dir(&cwd)
            .output();

        let new_files: Vec<String> = if let Ok(output) = &added_output {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .split('\n')
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()
        } else {
            vec![]
        };

        // 4. Restore all tracked files to checkpoint state
        let _ = Command::new("git")
            .args(["checkout", &info.sha, "--", "."])
            .current_dir(&cwd)
            .output();

        // 5. Unstage everything
        let _ = Command::new("git")
            .args(["reset"])
            .current_dir(&cwd)
            .output();

        // 6. Delete agent-created files (they're captured in agent_sha)
        for new_file in &new_files {
            let path = Path::new(&cwd).join(new_file);
            let _ = fs::remove_file(&path);
            // Clean up empty parent dirs
            if let Some(parent) = path.parent() {
                let _ = fs::remove_dir(parent); // only removes if empty
            }
        }

        // 7. Update checkpoint meta
        info.agent_sha = agent_sha;
        info.stashed = true;
        save_checkpoint_meta(&info)?;

        Ok(true)
    } else {
        // Non-git: copy current files to agent staging dir, restore from checkpoint
        let agent_dir = reviews_dir().join(&checkpoint_id).join("agent");
        fs::create_dir_all(&agent_dir).map_err(|e| e.to_string())?;

        let files = collect_files(&cwd)?;
        for rel_path in &files {
            let src = Path::new(&cwd).join(rel_path);
            let dst = agent_dir.join(rel_path);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let _ = fs::copy(&src, &dst);
        }

        // Restore checkpoint files to disk
        let snap_dir = checkpoint_files_dir(&checkpoint_id);
        if snap_dir.exists() {
            let snap_files = collect_files(&snap_dir.to_string_lossy())?;
            for rel_path in &snap_files {
                let src = snap_dir.join(rel_path);
                let dst = Path::new(&cwd).join(rel_path);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let _ = fs::copy(&src, &dst);
            }
            // Delete agent-created files
            for rel_path in &files {
                if !snap_files.contains(rel_path) {
                    let path = Path::new(&cwd).join(rel_path);
                    let _ = fs::remove_file(&path);
                }
            }
        }

        info.stashed = true;
        save_checkpoint_meta(&info)?;

        // Check if there are actual differences
        let diffs = scan_non_git_changes(
            &snap_dir.to_string_lossy(),
            &agent_dir,
        );
        Ok(diffs.map(|d| !d.is_empty()).unwrap_or(false))
    }
}

/// Diff checkpoint_sha vs agent_sha (both in git object store — not touching disk)
#[tauri::command]
pub fn scan_stashed_changes(cwd: String, checkpoint_id: String) -> Result<Vec<FileDiff>, String> {
    let info = load_checkpoint_meta(&checkpoint_id)?;

    if info.is_git {
        if info.agent_sha.is_empty() {
            return Ok(vec![]);
        }
        Ok(parse_git_diffs_between(&cwd, &info.sha, &info.agent_sha))
    } else {
        let snap_dir = checkpoint_files_dir(&checkpoint_id);
        let agent_dir = reviews_dir().join(&checkpoint_id).join("agent");
        if agent_dir.exists() && snap_dir.exists() {
            scan_non_git_changes(&snap_dir.to_string_lossy(), &agent_dir)
        } else {
            Ok(vec![])
        }
    }
}

/// Read a file's content from the agent's stashed state
#[tauri::command]
pub fn get_file_from_agent_stash(cwd: String, checkpoint_id: String, file_path: String) -> Result<String, String> {
    let info = load_checkpoint_meta(&checkpoint_id)?;

    if info.is_git {
        if info.agent_sha.is_empty() {
            return Ok(String::new());
        }
        let repo_root = git_repo_root(&cwd)?;
        let abs_path = if Path::new(&file_path).is_absolute() {
            PathBuf::from(&file_path)
        } else {
            Path::new(&cwd).join(&file_path)
        };
        let rel_path = abs_path
            .strip_prefix(&repo_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.clone());

        let output = Command::new("git")
            .args(["show", &format!("{}:{}", info.agent_sha, rel_path)])
            .current_dir(&cwd)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // File was deleted by agent
            Ok(String::new())
        }
    } else {
        let agent_dir = reviews_dir().join(&checkpoint_id).join("agent");
        let agent_file = agent_dir.join(&file_path);
        if agent_file.exists() {
            fs::read_to_string(&agent_file).map_err(|e| e.to_string())
        } else {
            Ok(String::new())
        }
    }
}

/// Write one accepted file from agent stash to disk
#[tauri::command]
pub fn apply_agent_file(cwd: String, checkpoint_id: String, file_path: String, status: String) -> Result<(), String> {
    if status == "D" {
        // Agent deleted this file — delete it from disk
        let full = Path::new(&cwd).join(&file_path);
        if full.exists() {
            fs::remove_file(&full).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    // Get content from agent stash and write to disk
    let content = get_file_from_agent_stash(cwd.clone(), checkpoint_id, file_path.clone())?;
    let full = Path::new(&cwd).join(&file_path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&full, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Finalize review: apply accepted files, save record, clean up
#[tauri::command]
pub fn finalize_review(
    cwd: String,
    checkpoint_id: String,
    decisions: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let info = load_checkpoint_meta(&checkpoint_id)?;

    // Scan the diffs to know file statuses
    let diffs = if info.stashed {
        if info.is_git && !info.agent_sha.is_empty() {
            parse_git_diffs_between(&cwd, &info.sha, &info.agent_sha)
        } else {
            let snap_dir = checkpoint_files_dir(&checkpoint_id);
            let agent_dir = reviews_dir().join(&checkpoint_id).join("agent");
            if agent_dir.exists() && snap_dir.exists() {
                scan_non_git_changes(&snap_dir.to_string_lossy(), &agent_dir).unwrap_or_default()
            } else {
                vec![]
            }
        }
    } else {
        vec![]
    };

    let mut reviewed_files = Vec::new();

    for diff in &diffs {
        let decision = decisions.get(&diff.path).map(|s| s.as_str()).unwrap_or("rejected");
        if decision == "accepted" {
            apply_agent_file(cwd.clone(), checkpoint_id.clone(), diff.path.clone(), diff.status.clone())?;
        }
        reviewed_files.push(ReviewedFile {
            path: diff.path.clone(),
            status: diff.status.clone(),
            decision: decision.to_string(),
        });
    }

    // Save review record
    let record = ReviewRecord {
        checkpoint_id: checkpoint_id.clone(),
        cwd: cwd.clone(),
        created_at: info.created_at.clone(),
        completed_at: chrono::Utc::now().to_rfc3339(),
        files: reviewed_files,
    };
    let reviews = reviews_dir();
    fs::create_dir_all(&reviews).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(reviews.join(format!("{}.json", checkpoint_id)), json).map_err(|e| e.to_string())?;

    // Clean up checkpoint meta and files
    let _ = fs::remove_file(checkpoint_meta_path(&checkpoint_id));
    let _ = fs::remove_dir_all(checkpoint_files_dir(&checkpoint_id));
    let _ = fs::remove_dir_all(reviews_dir().join(&checkpoint_id));

    Ok(())
}

/// List past review records
#[tauri::command]
pub fn list_reviews(cwd: String) -> Result<Vec<ReviewRecord>, String> {
    let dir = reviews_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut records = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(record) = serde_json::from_str::<ReviewRecord>(&data) {
                    if record.cwd == cwd {
                        records.push(record);
                    }
                }
            }
        }
    }
    records.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));
    Ok(records)
}

/// Scan working tree vs checkpoint (for live polling while agent is running)
#[tauri::command]
pub fn scan_changes(cwd: String, checkpoint_id: Option<String>) -> Result<Vec<FileDiff>, String> {
    if !is_git_repo(&cwd) {
        if let Some(ref cp_id) = checkpoint_id {
            let snap_dir = checkpoint_files_dir(cp_id);
            if snap_dir.exists() {
                return scan_non_git_changes(&cwd, &snap_dir);
            }
        }
        return Ok(vec![]);
    }

    let sha = checkpoint_id
        .as_deref()
        .and_then(|id| resolve_diff_sha(&cwd, id))
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&cwd)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        });

    match sha {
        Some(sha) if !sha.is_empty() => Ok(parse_git_diffs(&cwd, &sha)),
        _ => {
            let mut diffs = Vec::new();
            if let Ok(output) = Command::new("git")
                .args(["ls-files", "--others", "--exclude-standard"])
                .current_dir(&cwd)
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.trim().split('\n') {
                    let line = line.trim();
                    if !line.is_empty() {
                        diffs.push(FileDiff {
                            path: line.to_string(),
                            status: "A".to_string(),
                            additions: 0,
                            deletions: 0,
                        });
                    }
                }
            }
            Ok(diffs)
        }
    }
}

#[tauri::command]
pub fn get_checkpoint_summary(cwd: String, checkpoint_id: String) -> Result<Vec<FileDiff>, String> {
    scan_changes(cwd, Some(checkpoint_id))
}

#[tauri::command]
pub fn get_file_diff(cwd: String, checkpoint_id: String, file_path: String) -> Result<String, String> {
    let sha = resolve_diff_sha(&cwd, &checkpoint_id);
    if is_git_repo(&cwd) {
        if let Some(ref sha) = sha {
            let output = Command::new("git")
                .args(["diff", sha, "--", &file_path])
                .current_dir(&cwd)
                .output()
                .map_err(|e| e.to_string())?;
            let diff_text = String::from_utf8_lossy(&output.stdout).to_string();
            if !diff_text.is_empty() {
                return Ok(diff_text);
            }
        }
        let full_path = Path::new(&cwd).join(&file_path);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;
            let lines: Vec<String> = content.lines().map(|l| format!("+{}", l)).collect();
            return Ok(format!("--- /dev/null\n+++ b/{}\n@@ -0,0 +1,{} @@\n{}", file_path, lines.len(), lines.join("\n")));
        }
        Ok(String::new())
    } else {
        let snap_dir = checkpoint_files_dir(&checkpoint_id);
        let snap_file = snap_dir.join(&file_path);
        let cur_file = Path::new(&cwd).join(&file_path);
        let snap_content = fs::read_to_string(&snap_file).unwrap_or_default();
        let cur_content = fs::read_to_string(&cur_file).unwrap_or_default();
        Ok(format!(
            "--- a/{}\n+++ b/{}\n\nOriginal ({} lines) -> Current ({} lines)",
            file_path, file_path,
            snap_content.lines().count(),
            cur_content.lines().count()
        ))
    }
}

#[tauri::command]
pub fn get_file_at_checkpoint(cwd: String, checkpoint_id: String, file_path: String) -> Result<String, String> {
    if is_git_repo(&cwd) {
        let sha = resolve_diff_sha(&cwd, &checkpoint_id);
        if let Some(sha) = sha {
            let repo_root = git_repo_root(&cwd)?;
            let abs_path = if Path::new(&file_path).is_absolute() {
                PathBuf::from(&file_path)
            } else {
                Path::new(&cwd).join(&file_path)
            };
            let rel_path = abs_path
                .strip_prefix(&repo_root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file_path.clone());

            let output = Command::new("git")
                .args(["show", &format!("{}:{}", sha, rel_path)])
                .current_dir(&cwd)
                .output()
                .map_err(|e| e.to_string())?;

            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        Ok(String::new())
    } else {
        let snap_dir = checkpoint_files_dir(&checkpoint_id);
        let snap_file = snap_dir.join(&file_path);
        if snap_file.exists() {
            fs::read_to_string(&snap_file).map_err(|e| e.to_string())
        } else {
            Ok(String::new())
        }
    }
}

#[tauri::command]
pub fn revert_file(cwd: String, checkpoint_id: String, file_path: String) -> Result<(), String> {
    if is_git_repo(&cwd) {
        let sha = resolve_diff_sha(&cwd, &checkpoint_id);
        if let Some(sha) = sha {
            let output = Command::new("git")
                .args(["checkout", &sha, "--", &file_path])
                .current_dir(&cwd)
                .output()
                .map_err(|e| e.to_string())?;
            if output.status.success() {
                let _ = Command::new("git")
                    .args(["reset", "HEAD", "--", &file_path])
                    .current_dir(&cwd)
                    .output();
                return Ok(());
            }
        }
        let full_path = Path::new(&cwd).join(&file_path);
        if full_path.exists() {
            fs::remove_file(&full_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        let snap_dir = checkpoint_files_dir(&checkpoint_id);
        let snap_file = snap_dir.join(&file_path);
        let cur_file = Path::new(&cwd).join(&file_path);
        if snap_file.exists() {
            let content = fs::read_to_string(&snap_file).map_err(|e| e.to_string())?;
            if let Some(parent) = cur_file.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&cur_file, content).map_err(|e| e.to_string())?;
        } else if cur_file.exists() {
            fs::remove_file(&cur_file).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[tauri::command]
pub fn revert_all(cwd: String, checkpoint_id: String) -> Result<(), String> {
    let diffs = scan_changes(cwd.clone(), Some(checkpoint_id.clone()))?;
    for diff in &diffs {
        revert_file(cwd.clone(), checkpoint_id.clone(), diff.path.clone())?;
    }
    Ok(())
}

#[tauri::command]
pub fn finalize_checkpoint(_cwd: String, checkpoint_id: String, action: String) -> Result<(), String> {
    match action.as_str() {
        "accept" => {
            let _ = fs::remove_file(checkpoint_meta_path(&checkpoint_id));
            let _ = fs::remove_dir_all(checkpoint_files_dir(&checkpoint_id));
        }
        _ => return Err(format!("Unknown action: {}", action)),
    }
    Ok(())
}
