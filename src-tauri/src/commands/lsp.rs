use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use log;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, RwLock};

use super::terminal::resolve_binary;

// ─── State ───────────────────────────────────────────────────────────────────

struct LspServer {
    id: String,
    language: String,
    stdin: Arc<Mutex<ChildStdin>>,
    child: Arc<Mutex<Child>>,
}

pub struct LspManager {
    servers: RwLock<HashMap<String, LspServer>>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
        }
    }
}

// ─── Serialisable response types ─────────────────────────────────────────────

#[derive(serde::Serialize, Clone)]
pub struct LspServerInfo {
    pub id: String,
    pub language: String,
}

#[derive(serde::Serialize, Clone)]
pub struct LspBinaryStatus {
    pub language: String,
    pub binary: String,
    pub found: bool,
    pub path: Option<String>,
    pub install_hint: Option<String>,
}

// ─── Server registry ─────────────────────────────────────────────────────────

struct ServerConfig {
    binary: &'static str,
    args: &'static [&'static str],
}

fn server_config(language: &str) -> Option<ServerConfig> {
    match language {
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => Some(ServerConfig {
            binary: "typescript-language-server",
            args: &["--stdio"],
        }),
        "rust" => Some(ServerConfig {
            binary: "rust-analyzer",
            args: &[],
        }),
        "python" => Some(ServerConfig {
            binary: "pyright-langserver",
            args: &["--stdio"],
        }),
        "go" => Some(ServerConfig {
            binary: "gopls",
            args: &[],
        }),
        "ruby" => Some(ServerConfig {
            binary: "ruby-lsp",
            args: &[],
        }),
        "svelte" => Some(ServerConfig {
            binary: "svelteserver",
            args: &["--stdio"],
        }),
        "css" | "scss" | "less" => Some(ServerConfig {
            binary: "vscode-css-language-server",
            args: &["--stdio"],
        }),
        "html" => Some(ServerConfig {
            binary: "vscode-html-language-server",
            args: &["--stdio"],
        }),
        "json" | "jsonc" => Some(ServerConfig {
            binary: "vscode-json-language-server",
            args: &["--stdio"],
        }),
        "yaml" => Some(ServerConfig {
            binary: "yaml-language-server",
            args: &["--stdio"],
        }),
        "toml" => Some(ServerConfig {
            binary: "taplo",
            args: &["lsp", "stdio"],
        }),
        "graphql" => Some(ServerConfig {
            binary: "graphql-lsp",
            args: &["server", "-m", "stream"],
        }),
        "elixir" => Some(ServerConfig {
            binary: "elixir-ls",
            args: &[],
        }),
        _ => None,
    }
}

fn install_hint(language: &str) -> &'static str {
    match language {
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => {
            "npm install -g typescript-language-server typescript"
        }
        "rust" => "rustup component add rust-analyzer",
        "python" => "npm install -g pyright",
        "go" => "go install golang.org/x/tools/gopls@latest",
        "ruby" => "gem install ruby-lsp",
        "svelte" => "npm install -g svelte-language-server",
        "css" | "scss" | "less" | "html" | "json" | "jsonc" => {
            "npm install -g vscode-langservers-extracted"
        }
        "yaml" => "npm install -g yaml-language-server",
        "toml" => "cargo install taplo-cli --features lsp",
        "graphql" => "npm install -g graphql-language-service-cli",
        "elixir" => "See https://github.com/elixir-lsp/elixir-ls for installation",
        _ => "See the language server documentation for installation instructions",
    }
}

// ─── Binary discovery ────────────────────────────────────────────────────────

/// Find a binary by checking common install dirs (via resolve_binary) then PATH.
fn find_binary(name: &str) -> Option<String> {
    // resolve_binary searches /usr/local/bin, homebrew, ~/.cargo/bin, etc.
    let resolved = resolve_binary(name);
    if resolved != name {
        return Some(resolved);
    }

    // Fall back to scanning PATH directories
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = std::path::Path::new(dir).join(name);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }

    None
}

// ─── LSP JSON-RPC framing ────────────────────────────────────────────────────

/// Read one LSP message from stdout. Returns `None` on clean EOF (server exited).
async fn read_lsp_message(
    reader: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<Option<String>, std::io::Error> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    // Read HTTP-style headers until blank line
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None); // EOF
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(val) = trimmed.strip_prefix("Content-Length: ") {
            content_length = val.trim().parse().ok();
        }
        // Content-Type and any other headers are intentionally ignored
    }

    let len = content_length.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "LSP: missing Content-Length header",
        )
    })?;

    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).await?;
    Ok(Some(String::from_utf8_lossy(&body).to_string()))
}

/// Write one LSP message to stdin with proper Content-Length framing.
async fn write_lsp_message(
    stdin: &mut ChildStdin,
    message: &str,
) -> Result<(), std::io::Error> {
    let header = format!("Content-Length: {}\r\n\r\n", message.len());
    stdin.write_all(header.as_bytes()).await?;
    stdin.write_all(message.as_bytes()).await?;
    stdin.flush().await
}

// ─── Tauri commands ──────────────────────────────────────────────────────────

/// Spawn a language server and stream its output to the frontend via `on_message`.
/// Returns a `server_id` the frontend uses for `lsp_send` / `lsp_stop`.
#[tauri::command]
pub async fn lsp_start(
    app: AppHandle,
    language: String,
    workspace_root: String,
    on_message: Channel<String>,
    state: tauri::State<'_, Arc<LspManager>>,
) -> Result<String, String> {
    let config = server_config(&language)
        .ok_or_else(|| format!("No language server configured for: {language}"))?;

    let binary = find_binary(config.binary).unwrap_or_else(|| config.binary.to_string());

    let mut child = tokio::process::Command::new(&binary)
        .args(config.args)
        .current_dir(&workspace_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {e}", config.binary))?;

    let stdin = child.stdin.take().ok_or("Failed to acquire stdin handle")?;
    let stdout = child.stdout.take().ok_or("Failed to acquire stdout handle")?;

    let server_id = uuid::Uuid::new_v4().to_string();
    let sid = server_id.clone();

    // Background task: stream stdout frames → Tauri Channel
    let app_handle = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        loop {
            match read_lsp_message(&mut reader).await {
                Ok(Some(msg)) => {
                    if on_message.send(msg).is_err() {
                        break; // frontend disconnected
                    }
                }
                Ok(None) => break, // server exited cleanly
                Err(e) => {
                    log::warn!("[lsp:{sid}] read error: {e}");
                    break;
                }
            }
        }
        let _ = app_handle.emit(&format!("lsp-server-closed-{sid}"), ());
        log::info!("[lsp:{sid}] server process exited");
    });

    let server = LspServer {
        id: server_id.clone(),
        language: language.clone(),
        stdin: Arc::new(Mutex::new(stdin)),
        child: Arc::new(Mutex::new(child)),
    };

    state.servers.write().await.insert(server_id.clone(), server);
    log::info!("[lsp] started '{language}' server (id={server_id})");
    Ok(server_id)
}

/// Send a JSON-RPC message to a running language server's stdin.
#[tauri::command]
pub async fn lsp_send(
    server_id: String,
    message: String,
    state: tauri::State<'_, Arc<LspManager>>,
) -> Result<(), String> {
    let servers = state.servers.read().await;
    let server = servers
        .get(&server_id)
        .ok_or_else(|| format!("No LSP server with id: {server_id}"))?;

    let mut stdin = server.stdin.lock().await;
    write_lsp_message(&mut stdin, &message)
        .await
        .map_err(|e| format!("Failed to write LSP message: {e}"))
}

/// Kill a language server and remove it from the manager.
#[tauri::command]
pub async fn lsp_stop(
    server_id: String,
    state: tauri::State<'_, Arc<LspManager>>,
) -> Result<(), String> {
    let mut servers = state.servers.write().await;
    if let Some(server) = servers.remove(&server_id) {
        let mut child = server.child.lock().await;
        child.kill().await.ok();
        log::info!("[lsp] stopped server (id={server_id})");
    }
    Ok(())
}

/// Kill all running language servers. Called on project close / window teardown.
#[tauri::command]
pub async fn lsp_stop_all(
    state: tauri::State<'_, Arc<LspManager>>,
) -> Result<(), String> {
    let mut servers = state.servers.write().await;
    for (id, server) in servers.drain() {
        let mut child = server.child.lock().await;
        child.kill().await.ok();
        log::info!("[lsp] stopped server (id={id})");
    }
    Ok(())
}

/// List all currently running language servers.
#[tauri::command]
pub async fn lsp_list(
    state: tauri::State<'_, Arc<LspManager>>,
) -> Result<Vec<LspServerInfo>, String> {
    let servers = state.servers.read().await;
    Ok(servers
        .values()
        .map(|s| LspServerInfo {
            id: s.id.clone(),
            language: s.language.clone(),
        })
        .collect())
}

/// Check whether a language server binary is installed on this machine.
#[tauri::command]
pub async fn lsp_check_binary(language: String) -> Result<LspBinaryStatus, String> {
    let config = server_config(&language)
        .ok_or_else(|| format!("Unknown language: {language}"))?;

    let path = find_binary(config.binary);
    let found = path.is_some();
    let hint = if found {
        None
    } else {
        Some(install_hint(&language).to_string())
    };

    Ok(LspBinaryStatus {
        language,
        binary: config.binary.to_string(),
        found,
        path,
        install_hint: hint,
    })
}
