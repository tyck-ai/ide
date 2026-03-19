use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub file_path: String,
    pub line_number: u32,
    pub column: u32,
    pub line_text: String,
    pub match_start: u32,
    pub match_end: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub matches: Vec<SearchMatch>,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceResult {
    pub files_changed: u32,
    pub replacements_made: u32,
}

// Directories to skip when walking
const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", "dist", ".next", "__pycache__",
    ".svelte-kit", "build", ".cache", "out", ".turbo",
];

/// Very small glob matcher supporting `*` (any chars except `/`) and `**` (any path segment).
fn glob_matches(pattern: &str, path: &str) -> bool {
    let filename = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path);

    // If the pattern has no path separator, match against filename only
    if !pattern.contains('/') {
        return simple_glob(pattern, filename);
    }
    // Otherwise match the full relative path
    simple_glob(pattern, path)
}

fn simple_glob(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match(&pat, &txt)
}

fn glob_match(pat: &[char], txt: &[char]) -> bool {
    match (pat.first(), txt.first()) {
        (None, None) => true,
        (Some('*'), _) if pat.get(1) == Some(&'*') => {
            // `**` matches any sequence including path separators
            glob_match(&pat[2..], txt)
                || (!txt.is_empty() && glob_match(pat, &txt[1..]))
        }
        (Some('*'), _) => {
            // `*` matches any sequence except `/`
            if txt.is_empty() {
                return glob_match(&pat[1..], txt);
            }
            glob_match(&pat[1..], txt)
                || (txt[0] != '/' && glob_match(pat, &txt[1..]))
        }
        (None, _) | (_, None) => false,
        (Some(p), Some(t)) => {
            if p == t { glob_match(&pat[1..], &txt[1..]) } else { false }
        }
    }
}

fn search_file(
    abs_path: &Path,
    rel_path: &str,
    query: &str,
    case_sensitive: bool,
    max_matches: &mut usize,
    results: &mut Vec<SearchMatch>,
) {
    if *max_matches == 0 {
        return;
    }
    let Ok(content) = std::fs::read_to_string(abs_path) else { return };
    // Skip binary files (null bytes)
    if content.contains('\0') {
        return;
    }

    let needle = if case_sensitive { query.to_string() } else { query.to_lowercase() };

    for (line_idx, line) in content.lines().enumerate() {
        let haystack = if case_sensitive { line.to_string() } else { line.to_lowercase() };
        let mut search_from = 0usize;
        while let Some(pos) = haystack[search_from..].find(&needle) {
            let match_start = search_from + pos;
            let match_end = match_start + query.len();
            results.push(SearchMatch {
                file_path: rel_path.to_string(),
                line_number: (line_idx + 1) as u32,
                column: (match_start + 1) as u32,
                line_text: line.trim_end_matches('\n').to_string(),
                match_start: match_start as u32,
                match_end: match_end as u32,
            });
            *max_matches -= 1;
            if *max_matches == 0 {
                return;
            }
            search_from = match_start + needle.len().max(1);
        }
    }
}

fn walk_dir(
    dir: &Path,
    root: &str,
    query: &str,
    case_sensitive: bool,
    include_glob: &str,
    exclude_glob: &str,
    max_matches: &mut usize,
    results: &mut Vec<SearchMatch>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden files/dirs and known noise dirs
        if name_str.starts_with('.') {
            // Allow .env files etc but skip .git, .cache, etc.
            if path.is_dir() {
                continue;
            }
        }

        if path.is_dir() {
            if SKIP_DIRS.contains(&name_str.as_ref()) {
                continue;
            }
            walk_dir(&path, root, query, case_sensitive, include_glob, exclude_glob, max_matches, results);
        } else {
            let rel_path = path.to_string_lossy();
            let rel = if rel_path.starts_with(root) {
                rel_path[root.len()..].trim_start_matches('/').to_string()
            } else {
                rel_path.to_string()
            };

            // Apply include glob filter
            if !include_glob.is_empty() && !glob_matches(include_glob, &rel) {
                continue;
            }
            // Apply exclude glob filter
            if !exclude_glob.is_empty() && glob_matches(exclude_glob, &rel) {
                continue;
            }

            search_file(&path, &rel, query, case_sensitive, max_matches, results);
        }

        if *max_matches == 0 {
            return;
        }
    }
}

#[tauri::command]
pub fn search_in_project(
    root: String,
    query: String,
    _is_regex: bool,
    case_sensitive: bool,
    include_glob: String,
    exclude_glob: String,
) -> Result<SearchResult, String> {
    if query.is_empty() {
        return Ok(SearchResult { matches: vec![], truncated: false });
    }

    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("Project root not found: {}", root));
    }

    let mut results = Vec::new();
    let mut max_matches = 500usize;

    walk_dir(
        root_path,
        &root,
        &query,
        case_sensitive,
        &include_glob,
        &exclude_glob,
        &mut max_matches,
        &mut results,
    );

    let truncated = max_matches == 0;
    Ok(SearchResult { matches: results, truncated })
}

#[tauri::command]
pub fn replace_in_project(
    root: String,
    query: String,
    replacement: String,
    _is_regex: bool,
    case_sensitive: bool,
    file_paths: Vec<String>,
) -> Result<ReplaceResult, String> {
    if query.is_empty() || file_paths.is_empty() {
        return Ok(ReplaceResult { files_changed: 0, replacements_made: 0 });
    }

    let mut files_changed = 0u32;
    let mut replacements_made = 0u32;

    for rel_path in &file_paths {
        let abs_path = format!("{}/{}", root, rel_path);
        // Guard: reject any path that resolves outside the root (path traversal)
        let canonical = std::fs::canonicalize(&abs_path)
            .map_err(|e| format!("Failed to resolve {}: {}", rel_path, e))?;
        let root_canonical = std::fs::canonicalize(&root)
            .map_err(|e| format!("Failed to resolve root: {}", e))?;
        if !canonical.starts_with(&root_canonical) {
            return Err(format!("Path escapes root: {}", rel_path));
        }
        let content = std::fs::read_to_string(&canonical)
            .map_err(|e| format!("Failed to read {}: {}", rel_path, e))?;

        let new_content = if case_sensitive {
            let count = content.matches(query.as_str()).count() as u32;
            if count == 0 { continue; }
            replacements_made += count;
            content.replace(query.as_str(), &replacement)
        } else {
            let lower_content = content.to_lowercase();
            let lower_query = query.to_lowercase();
            let count = lower_content.matches(lower_query.as_str()).count() as u32;
            if count == 0 { continue; }
            replacements_made += count;
            let mut result = String::with_capacity(content.len());
            let mut last = 0usize;
            let qlen = query.len();
            for (start, _) in lower_content.match_indices(lower_query.as_str()) {
                result.push_str(&content[last..start]);
                result.push_str(&replacement);
                last = start + qlen;
            }
            result.push_str(&content[last..]);
            result
        };

        std::fs::write(&canonical, &new_content)
            .map_err(|e| format!("Failed to write {}: {}", rel_path, e))?;
        files_changed += 1;
    }

    Ok(ReplaceResult { files_changed, replacements_made })
}
