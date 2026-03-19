//! Minimal MCP (Model Context Protocol) server with SSE transport.
//!
//! Exposes one tool: `push_and_create_pr`.  When the agent calls it the server
//! emits a `push-pr-requested` Tauri event and returns immediately, letting the
//! user confirm the operation in the IDE UI.

use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, StreamExt};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Port registry — start_agent reads this to build the --mcp-config argument.
// ---------------------------------------------------------------------------

static MCP_PORT: OnceCell<u16> = OnceCell::new();

pub fn get_mcp_port() -> Option<u16> {
    MCP_PORT.get().copied()
}

// ---------------------------------------------------------------------------
// Shared server state
// ---------------------------------------------------------------------------

struct McpState {
    app: AppHandle,
    /// Active SSE sessions: session_id → channel to push events back to client.
    sessions: Mutex<HashMap<String, mpsc::Sender<Event>>>,
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

pub async fn start_mcp_server(app: AppHandle) -> anyhow::Result<u16> {
    // Target the 55000-58999 range — well above every common dev port (3000, 5000,
    // 8080, …) and safely below the IANA ephemeral ceiling of 65535.
    // Falls back to :0 (OS-assigned ephemeral) if all attempts fail.
    let listener = {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u16;
        let mut found = None;
        for i in 0u16..30 {
            let port = 55000 + seed.wrapping_add(i.wrapping_mul(97)) % 4000;
            if let Ok(l) = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                found = Some(l);
                break;
            }
        }
        match found {
            Some(l) => l,
            None => tokio::net::TcpListener::bind("127.0.0.1:0").await?,
        }
    };
    let port = listener.local_addr()?.port();
    let _ = MCP_PORT.set(port);

    let state = Arc::new(McpState {
        app,
        sessions: Mutex::new(HashMap::new()),
    });

    let router = Router::new()
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .with_state(state);

    tokio::spawn(async move {
        axum::serve(listener, router).await.ok();
    });

    log::info!("[mcp] Server listening on http://127.0.0.1:{}/sse", port);
    Ok(port)
}

// ---------------------------------------------------------------------------
// SSE endpoint — client connects here, gets an endpoint URL back
// ---------------------------------------------------------------------------

async fn sse_handler(
    State(state): State<Arc<McpState>>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let session_id = Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel::<Event>(32);
    state.sessions.lock().await.insert(session_id.clone(), tx);

    // First event tells the client where to POST messages for this session.
    let endpoint_event = Event::default()
        .event("endpoint")
        .data(format!("/message?sessionId={}", session_id));

    let initial = stream::once(async move { Ok::<Event, Infallible>(endpoint_event) });
    let relay = ReceiverStream::new(rx).map(|e| Ok::<Event, Infallible>(e));

    Sse::new(initial.chain(relay)).keep_alive(KeepAlive::default())
}

// ---------------------------------------------------------------------------
// Message endpoint — client POSTs JSON-RPC requests here
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct MessageQuery {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

async fn message_handler(
    State(state): State<Arc<McpState>>,
    Query(params): Query<MessageQuery>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let session_id = params.session_id.unwrap_or_default();
    let id = req.get("id").cloned();

    // Notifications have no `id` — no response needed.
    if let Some(id) = id {
        let result = dispatch_request(&req, &state.app).await;
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        });

        let sessions = state.sessions.lock().await;
        if let Some(tx) = sessions.get(&session_id) {
            let event = Event::default()
                .event("message")
                .data(response.to_string());
            let _ = tx.send(event).await;
        }
    }

    axum::http::StatusCode::ACCEPTED
}

// ---------------------------------------------------------------------------
// JSON-RPC dispatch
// ---------------------------------------------------------------------------

async fn dispatch_request(req: &Value, app: &AppHandle) -> Value {
    let method = req["method"].as_str().unwrap_or("");

    match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "tyck-ide", "version": "1.0.0" }
        }),

        "tools/list" => json!({
            "tools": [{
                "name": "push_and_create_pr",
                "description": "Request the IDE to push the current agent branch and open the pull request dialog. The user will review any pending file changes before confirming. This tool returns immediately — the push/PR happens asynchronously after user approval.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Suggested PR title"
                        },
                        "body": {
                            "type": "string",
                            "description": "Suggested PR description summarising the changes made"
                        }
                    },
                    "required": []
                }
            }]
        }),

        "tools/call" => {
            let tool = req["params"]["name"].as_str().unwrap_or("");
            if tool == "push_and_create_pr" {
                let args = req["params"]["arguments"].clone();
                let _ = app.emit("push-pr-requested", &args);
                log::info!("[mcp] push_and_create_pr requested: {:?}", args);
                json!({
                    "content": [{
                        "type": "text",
                        "text": "PR request received. The user will review pending changes in the IDE and confirm before pushing."
                    }],
                    "isError": false
                })
            } else {
                json!({
                    "content": [{ "type": "text", "text": format!("Unknown tool: {}", tool) }],
                    "isError": true
                })
            }
        }

        // Ignore other methods (ping, notifications/initialized, etc.)
        _ => json!(null),
    }
}
