use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: String,
    pub first_message: String,
    pub timestamp: String,
    pub model: String,
    pub is_active: bool,
    pub provider: String,
    pub provider_name: String,
    /// Human-readable slug (from Claude Code JSONL)
    #[serde(default)]
    pub slug: String,
    /// Full path to the original session file (for resume support)
    #[serde(default)]
    pub session_path: String,
}

// ── Claude Code native session discovery ──

/// Convert a cwd to the Claude Code project hash (e.g. /Users/foo/bar → -Users-foo-bar)
/// Claude Code replaces /, spaces, and dots with dashes.
fn claude_project_hash(cwd: &str) -> String {
    cwd.replace('/', "-").replace(' ', "-").replace('.', "-")
}

fn claude_projects_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("projects")
}

/// Discover all Claude Code sessions for a given project cwd by scanning
/// ~/.claude/projects/{hash}/*.jsonl files. Also includes sessions that ran
/// in worktrees created for this project.
pub fn discover_claude_sessions(cwd: &str) -> Vec<SessionInfo> {
    let projects_dir = claude_projects_dir();
    let valid_cwds = collect_valid_cwds(cwd);

    // Convert each valid cwd to its Claude project hash and scan that directory
    let mut sessions = Vec::new();

    for valid_cwd in &valid_cwds {
        let hash = claude_project_hash(valid_cwd);
        let dir = projects_dir.join(&hash);
        if !dir.exists() {
            continue;
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !name.ends_with(".jsonl") || path.is_dir() {
                continue;
            }

            let session_id = name.trim_end_matches(".jsonl").to_string();
            if session_id.is_empty() {
                continue;
            }

            let session_path = path.to_string_lossy().to_string();
            let info = parse_session_metadata(&path, &session_id, &session_path);
            sessions.push(info);
        }
    }

    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sessions
}

/// Parse the first lines of a JSONL session file to extract metadata.
fn parse_session_metadata(path: &PathBuf, session_id: &str, session_path: &str) -> SessionInfo {
    let mut first_message = String::new();
    let mut timestamp = String::new();
    let mut slug = String::new();
    let model = String::new();
    if let Ok(file) = fs::File::open(path) {
        let reader = BufReader::new(file);
        let mut lines_read = 0;

        for line in reader.lines() {
            if lines_read > 30 {
                break; // Only scan first 30 lines for metadata
            }
            lines_read += 1;

            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            let val: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Grab timestamp from earliest entry
            if timestamp.is_empty() {
                if let Some(ts) = val["timestamp"].as_str() {
                    timestamp = ts.to_string();
                } else if let Some(ts) = val["timestamp"].as_f64() {
                    // Unix milliseconds — convert to ISO
                    let secs = (ts / 1000.0) as i64;
                    if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
                        timestamp = dt.to_rfc3339();
                    }
                }
            }

            // Grab slug
            if slug.is_empty() {
                if let Some(s) = val["slug"].as_str() {
                    slug = s.to_string();
                }
            }

            // Grab first user message
            if first_message.is_empty() {
                let msg_type = val["type"].as_str().unwrap_or("");
                if msg_type == "user" {
                    if let Some(content) = val["message"]["content"].as_str() {
                        first_message = content.chars().take(120).collect();
                    }
                }
                // Also check queue-operation enqueue (has content field directly)
                if msg_type == "queue-operation" {
                    if let Some(content) = val["content"].as_str() {
                        first_message = content.chars().take(120).collect();
                    }
                }
            }

            // If we have everything, stop early
            if !first_message.is_empty() && !timestamp.is_empty() {
                // Keep scanning for slug/model if missing, but not too long
                if !slug.is_empty() || lines_read > 15 {
                    break;
                }
            }
        }
    }

    // Fallback: use file modification time as timestamp
    if timestamp.is_empty() {
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                let dt: chrono::DateTime<chrono::Utc> = modified.into();
                timestamp = dt.to_rfc3339();
            }
        }
    }

    if first_message.is_empty() {
        first_message = format!("Session {}", &session_id[..8.min(session_id.len())]);
    }

    SessionInfo {
        session_id: session_id.to_string(),
        first_message,
        timestamp,
        model,
        is_active: false,
        provider: "claude-code".to_string(),
        provider_name: "Claude Code".to_string(),
        slug,
        session_path: session_path.to_string(),
    }
}

// ── Shared: collect all valid cwds for a project (main + worktree paths) ──

/// Given a project cwd, return a set containing the cwd itself plus the worktree
/// paths of any tyck worktrees created for that project.
fn collect_valid_cwds(cwd: &str) -> std::collections::HashSet<String> {
    let mut cwds = std::collections::HashSet::new();
    cwds.insert(cwd.to_string());

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    
    // Method 1: Scan worktree metadata files in ~/.tyck/worktrees/
    let worktrees_dir = PathBuf::from(&home).join(".tyck").join("worktrees");

    if worktrees_dir.exists() {
        if let Ok(entries) = fs::read_dir(&worktrees_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&data) {
                        let main_cwd = meta["mainCwd"].as_str().unwrap_or("");
                        if main_cwd == cwd {
                            if let Some(wt_path) = meta["worktreePath"].as_str() {
                                cwds.insert(wt_path.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Method 2: Scan Claude's projects directory for worktree-named folders
    // This catches orphaned sessions where the worktree metadata was deleted
    // but Claude's session files remain.
    let projects_dir = claude_projects_dir();
    
    // Look for directories matching the pattern: -<cwd-hash-prefix>-*-worktrees-*
    // Claude uses the full path converted to hash, so we look for worktree patterns
    if projects_dir.exists() {
        if let Ok(entries) = fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                // Check if this is a worktree directory for the current cwd
                // Pattern: -Users-...-<cwd-path>-...-worktrees-<session-id>
                if name.contains("-worktrees-") {
                    let cwd_sanitized = cwd.replace('/', "-").trim_start_matches('-').to_string();
                    let name_trimmed = name.trim_start_matches('-');
                    
                    // Check if this worktree belongs to our cwd by checking if the
                    // pre-worktree part of the path matches our cwd
                    if let Some(wt_idx) = name_trimmed.find("-worktrees-") {
                        let prefix = &name_trimmed[..wt_idx];
                        if cwd_sanitized.ends_with(prefix) || prefix.ends_with(&cwd_sanitized) {
                            // Extract session ID from the worktree name
                            let session_id = &name_trimmed[wt_idx + 11..]; // "-worktrees-".len() = 11
                            // Reconstruct the worktree path
                            let wt_path = PathBuf::from(&home)
                                .join(".tyck")
                                .join("worktrees")
                                .join(session_id);
                            cwds.insert(wt_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    cwds
}

// ── Codex native session discovery ──

fn codex_home() -> PathBuf {
    if let Ok(h) = std::env::var("CODEX_HOME") {
        return PathBuf::from(h);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".codex")
}

/// Discover Codex sessions for a given cwd.
/// Codex stores sessions globally at ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
/// with cwd in the session_meta payload. We also use ~/.codex/history.jsonl as a
/// fast index to avoid scanning all rollout files.
pub fn discover_codex_sessions(cwd: &str) -> Vec<SessionInfo> {
    let codex = codex_home();

    let history_path = codex.join("history.jsonl");
    let sessions_dir = codex.join("sessions");

    if !sessions_dir.exists() {
        return vec![];
    }

    // Build set of valid cwds: main project + any worktree paths for this project
    let valid_cwds = collect_valid_cwds(cwd);

    // Build a map of session_id → (text, timestamp) from history.jsonl
    let mut history_index: std::collections::HashMap<String, (String, i64)> =
        std::collections::HashMap::new();
    if let Ok(file) = fs::File::open(&history_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                if let (Some(sid), Some(ts)) = (
                    val["session_id"].as_str(),
                    val["ts"].as_i64(),
                ) {
                    let text = val["text"].as_str().unwrap_or("").to_string();
                    history_index.insert(sid.to_string(), (text, ts));
                }
            }
        }
    }

    // Scan all rollout files — Codex organizes by date: YYYY/MM/DD/rollout-*.jsonl
    let mut sessions = Vec::new();
    scan_codex_dir(&sessions_dir, &valid_cwds, &history_index, &mut sessions);

    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sessions
}

/// Recursively scan Codex sessions directory for rollout-*.jsonl files.
fn scan_codex_dir(
    dir: &PathBuf,
    valid_cwds: &std::collections::HashSet<String>,
    history: &std::collections::HashMap<String, (String, i64)>,
    out: &mut Vec<SessionInfo>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_codex_dir(&path, valid_cwds, history, out);
            continue;
        }

        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if !name.starts_with("rollout-") || !name.ends_with(".jsonl") {
            continue;
        }

        if let Some(info) = parse_codex_rollout(&path, valid_cwds, history) {
            out.push(info);
        }
    }
}

/// Parse a Codex rollout file. Returns Some(SessionInfo) if the session's cwd matches.
fn parse_codex_rollout(
    path: &PathBuf,
    valid_cwds: &std::collections::HashSet<String>,
    history: &std::collections::HashMap<String, (String, i64)>,
) -> Option<SessionInfo> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // First line should be session_meta
    let first_line = lines.next()?.ok()?;
    let val: serde_json::Value = serde_json::from_str(&first_line).ok()?;

    if val["type"].as_str() != Some("session_meta") {
        return None;
    }

    let payload = &val["payload"];
    let session_cwd = payload["cwd"].as_str().unwrap_or("");

    // Only include sessions matching our project cwd (or its worktree paths)
    if !valid_cwds.contains(session_cwd) {
        return None;
    }

    let session_id = payload["id"].as_str().unwrap_or("").to_string();
    if session_id.is_empty() {
        return None;
    }

    let timestamp = payload["timestamp"].as_str().unwrap_or("").to_string();
    let model_provider = payload["model_provider"].as_str().unwrap_or("").to_string();

    // Try history index for the first message (fast, no need to scan full file)
    let first_message = if let Some((text, _)) = history.get(&session_id) {
        if text.is_empty() {
            extract_codex_first_message(&mut lines)
        } else {
            text.chars().take(120).collect()
        }
    } else {
        extract_codex_first_message(&mut lines)
    };

    let first_message = if first_message.is_empty() {
        format!("Session {}", &session_id[..8.min(session_id.len())])
    } else {
        first_message
    };

    Some(SessionInfo {
        session_id,
        first_message,
        timestamp,
        model: model_provider,
        is_active: false,
        provider: "codex".to_string(),
        provider_name: "Codex".to_string(),
        slug: String::new(),
        session_path: path.to_string_lossy().to_string(),
    })
}

/// Extract the first user message from remaining Codex rollout lines.
fn extract_codex_first_message(
    lines: &mut std::io::Lines<BufReader<fs::File>>,
) -> String {
    let mut scanned = 0;
    for line in lines {
        if scanned > 20 {
            break;
        }
        scanned += 1;

        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Codex user messages: type=response_item, payload.role=user
        if val["type"].as_str() == Some("response_item") {
            let payload = &val["payload"];
            if payload["role"].as_str() == Some("user") {
                if let Some(content) = payload["content"].as_array() {
                    for item in content {
                        if let Some(text) = item["text"].as_str() {
                            // Skip system/agent instructions (start with < or #)
                            let trimmed = text.trim();
                            if !trimmed.starts_with('<') && !trimmed.starts_with('#') && !trimmed.is_empty() {
                                return trimmed.chars().take(120).collect();
                            }
                        }
                    }
                }
            }
        }

        // Also check event_msg with user input
        if val["type"].as_str() == Some("event_msg") {
            if let Some(text) = val["payload"]["content"].as_str() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return trimmed.chars().take(120).collect();
                }
            }
        }
    }
    String::new()
}

// ── Unified session listing ──

/// List all discoverable sessions for the given cwd across all providers.
pub fn list_all_sessions(cwd: &str) -> Vec<SessionInfo> {
    let mut all = Vec::new();

    // Claude Code: native discovery
    all.extend(discover_claude_sessions(cwd));

    // Codex: native discovery
    all.extend(discover_codex_sessions(cwd));

    // TODO: Cursor, Copilot native discovery

    // Deduplicate by session_id (keep first occurrence, which will be from main cwd)
    let mut seen = std::collections::HashSet::new();
    all.retain(|s| seen.insert(s.session_id.clone()));

    // Sort by timestamp descending
    all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    all
}

// ── Resume args per provider ──

pub fn resume_args(provider: &str, session_id: &str) -> Vec<String> {
    match provider {
        "codex" => vec!["resume".into(), session_id.into()],
        _ => vec!["--resume".into(), session_id.into()],
    }
}

// ── Resume session preparation ──

/// Prepare a Claude Code session for resume by symlinking the original session file
/// to the new worktree's project hash directory. This allows Claude Code to find
/// the session history when running from a different worktree.
///
/// Returns Ok(()) on success, or an error message on failure.
pub fn prepare_claude_resume(
    original_session_path: &str,
    new_worktree_cwd: &str,
    session_id: &str,
) -> Result<(), String> {
    let original_path = PathBuf::from(original_session_path);
    
    let projects_dir = claude_projects_dir();
    let new_hash = claude_project_hash(new_worktree_cwd);
    let new_project_dir = projects_dir.join(&new_hash);
    let new_session_path = new_project_dir.join(format!("{}.jsonl", session_id));

    // If source and destination are the same, nothing to do
    // (session was originally created in this worktree)
    if original_path == new_session_path {
        return Ok(());
    }
    
    // Also check if they resolve to the same file (following symlinks)
    if let (Ok(orig_canon), Ok(new_canon)) = (original_path.canonicalize(), new_session_path.canonicalize()) {
        if orig_canon == new_canon {
            return Ok(());
        }
    }

    if !original_path.exists() {
        return Err(format!(
            "Original session file not found: {}",
            original_session_path
        ));
    }

    // Create the project directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&new_project_dir) {
        return Err(format!(
            "Failed to create project directory {}: {}",
            new_project_dir.display(),
            e
        ));
    }

    // If symlink already exists (pointing to same file), we're done
    if new_session_path.exists() {
        if let Ok(target) = fs::read_link(&new_session_path) {
            if target == original_path {
                return Ok(());
            }
        }
        // Remove existing file/symlink if it points elsewhere
        let _ = fs::remove_file(&new_session_path);
    }

    // Create symlink to original session file
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&original_path, &new_session_path).map_err(|e| {
            format!(
                "Failed to create symlink {} -> {}: {}",
                new_session_path.display(),
                original_path.display(),
                e
            )
        })?;
    }

    #[cfg(windows)]
    {
        // On Windows, use a file copy instead of symlink (symlinks require admin)
        fs::copy(&original_path, &new_session_path).map_err(|e| {
            format!(
                "Failed to copy session file {} -> {}: {}",
                original_path.display(),
                new_session_path.display(),
                e
            )
        })?;
    }

    Ok(())
}
