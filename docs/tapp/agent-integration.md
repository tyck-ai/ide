# Agent Integration Guide

Tapp extensions can deeply integrate with AI agents in Tyck IDE, providing tools that agents can invoke, hooks to intercept agent I/O, and the ability to spawn new agent sessions.

## Overview

| Feature | Description | Permission |
|---------|-------------|------------|
| **Tools** | Functions the agent can call | `agent:tools` |
| **Hooks** | Intercept agent input/output | `agent:hooks` |
| **Inject** | Send text to active session | `agent:inject` |
| **Spawn** | Create new agent sessions | `agent:spawn` |

## Tools

Tools are functions your app exposes that agents can discover and invoke.

### Defining Tools

```rust
use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct DatabaseExplorer {
    connections: Vec<Connection>,
}

impl App for DatabaseExplorer {
    // ... standard App methods
}

#[tapp::tools]
impl DatabaseExplorer {
    /// Execute a SQL query against the connected database
    #[tool(name = "query_database", description = "Execute SQL query and return results")]
    fn query_database(&mut self, args: serde_json::Value) -> ToolResult {
        let sql = args.get("sql")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'sql' parameter")?;
        
        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        match self.connections.first() {
            Some(conn) => {
                let rows = conn.query(sql, limit)?;
                ToolResult::json(serde_json::json!({
                    "rows": rows,
                    "count": rows.len()
                }))
            }
            None => ToolResult::error("No database connected")
        }
    }

    /// List all tables in the database
    #[tool(description = "List all tables in the connected database")]
    fn list_tables(&self, _args: serde_json::Value) -> ToolResult {
        match self.connections.first() {
            Some(conn) => {
                let tables = conn.list_tables()?;
                ToolResult::json(serde_json::json!({ "tables": tables }))
            }
            None => ToolResult::error("No database connected")
        }
    }

    /// Get schema for a specific table
    #[tool(description = "Get column definitions for a table")]
    fn get_table_schema(&self, args: serde_json::Value) -> ToolResult {
        let table = args.get("table")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'table' parameter")?;

        match self.connections.first() {
            Some(conn) => {
                let schema = conn.describe_table(table)?;
                ToolResult::json(serde_json::json!({ "columns": schema }))
            }
            None => ToolResult::error("No database connected")
        }
    }
}
```

### Tool Result Types

```rust
// Success with JSON data
ToolResult::json(serde_json::json!({ "key": "value" }))

// Success with no data
ToolResult::ok()

// Error message
ToolResult::error("Something went wrong")

// Success with text
ToolResult::text("Operation completed successfully")
```

### Tool Discovery

Tools are automatically discovered when your app is loaded. The agent sees:

```json
{
  "tools": [
    {
      "name": "query_database",
      "description": "Execute SQL query and return results",
      "parameters": {
        "type": "object",
        "properties": {
          "sql": { "type": "string", "description": "SQL query to execute" },
          "limit": { "type": "number", "description": "Max rows to return" }
        },
        "required": ["sql"]
      }
    }
  ]
}
```

### Required Permission

```json
{
  "permissions": ["agent:tools"]
}
```

## Hooks

Hooks let you intercept and modify agent communication.

### Available Hook Types

| Hook | Timing | Can Modify | Timeout |
|------|--------|------------|---------|
| `on_before_input` | Before agent receives input | Yes | 100ms |
| `on_after_output` | After agent produces output | No | 50ms |
| `on_tool_call` | When agent calls any tool | No | 50ms |
| `on_session_start` | When session begins | No | 50ms |
| `on_session_end` | When session ends | No | 50ms |

### Defining Hooks

```rust
#[tapp::hooks]
impl MyApp {
    /// Enrich user input with context
    #[hook(on_before_input)]
    fn enrich_input(&self, data: &serde_json::Value) -> HookResult {
        let input = data.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Add database context if query-related
        if input.to_lowercase().contains("database") 
            || input.to_lowercase().contains("sql") {
            let context = format!(
                "{}\n\n---\nAvailable databases: {}\nConnected to: {}",
                input,
                self.list_available_dbs(),
                self.current_connection_name()
            );
            HookResult::modify_input(context)
        } else {
            HookResult::pass_through()
        }
    }

    /// Log all agent outputs
    #[hook(on_after_output)]
    fn log_output(&self, data: &serde_json::Value) -> HookResult {
        if let Some(output) = data.get("output").and_then(|v| v.as_str()) {
            self.log_to_history(output);
        }
        HookResult::pass_through()
    }

    /// Track tool usage
    #[hook(on_tool_call)]
    fn track_tools(&self, data: &serde_json::Value) -> HookResult {
        if let Some(tool_name) = data.get("tool").and_then(|v| v.as_str()) {
            self.increment_tool_usage(tool_name);
        }
        HookResult::pass_through()
    }
}
```

### Hook Results

```rust
// Pass through without modification
HookResult::pass_through()

// Modify the input (only for on_before_input)
HookResult::modify_input("Modified text".to_string())

// Cancel the operation
HookResult::cancel()

// Default (same as pass_through)
HookResult::default()
```

### Required Permission

```json
{
  "permissions": ["agent:hooks"]
}
```

## Agent Session API

Your app can interact with agent sessions programmatically.

### Injecting Text

Send text to the currently active agent session:

```rust
use tapp::prelude::*;

impl MyApp {
    fn send_to_agent(&self, text: &str) -> Result<()> {
        AgentSession::inject(text)?;
        Ok(())
    }
}
```

**Permission**: `agent:inject`

### Spawning Sessions

Create new agent sessions:

```rust
impl MyApp {
    async fn run_background_task(&self, prompt: &str) -> Result<String> {
        let options = SpawnOptions {
            provider: Some("claude".to_string()),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            cwd: Some("/path/to/project".to_string()),
            visible: false, // Hidden session
        };

        let session = AgentSession::spawn(options)?;
        
        // Send prompt
        session.send(prompt)?;
        
        // Wait for response
        loop {
            if let Some(output) = session.get_output()? {
                session.kill()?;
                return Ok(output);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
```

**Permission**: `agent:spawn`

### Session Management

```rust
// Get session ID
let id = session.id();

// Send text
session.send("Hello")?;

// Check for output (non-blocking)
let output: Option<String> = session.get_output()?;

// Kill session
session.kill()?;
```

## Complete Example: Code Review Assistant

```rust
use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct CodeReviewAssistant {
    reviews: Vec<Review>,
    current_diff: Option<String>,
}

impl App for CodeReviewAssistant {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "load_diff" => {
                let diff: String = action.get("diff")?;
                self.current_diff = Some(diff);
                Ok(Response::render())
            }
            "request_review" => {
                if let Some(diff) = &self.current_diff {
                    AgentSession::inject(&format!(
                        "Please review this code diff and provide feedback:\n\n```diff\n{}\n```",
                        diff
                    ))?;
                }
                Ok(Response::ok())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::panel("Code Review").children([
            if let Some(diff) = &self.current_diff {
                ui::code(diff).language("diff").into()
            } else {
                ui::empty("Load a diff to review").into()
            },
            ui::button("Request AI Review")
                .primary()
                .on_click("request_review"),
        ])
    }
}

#[tapp::tools]
impl CodeReviewAssistant {
    #[tool(description = "Add a review comment to the current diff")]
    fn add_review_comment(&mut self, args: serde_json::Value) -> ToolResult {
        let line = args.get("line")
            .and_then(|v| v.as_u64())
            .ok_or("Missing 'line' parameter")? as usize;
        
        let comment = args.get("comment")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'comment' parameter")?;
        
        let severity = args.get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("info");

        self.reviews.push(Review {
            line,
            comment: comment.to_string(),
            severity: severity.to_string(),
        });

        ToolResult::ok()
    }

    #[tool(description = "Get all review comments")]
    fn get_review_comments(&self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({
            "reviews": self.reviews
        }))
    }
}

#[tapp::hooks]
impl CodeReviewAssistant {
    #[hook(on_before_input)]
    fn add_context(&self, data: &serde_json::Value) -> HookResult {
        let input = data.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if input.to_lowercase().contains("review") {
            if let Some(diff) = &self.current_diff {
                let enriched = format!(
                    "{}\n\nCurrent diff loaded ({} lines):\n```diff\n{}\n```",
                    input,
                    diff.lines().count(),
                    diff
                );
                return HookResult::modify_input(enriched);
            }
        }

        HookResult::pass_through()
    }
}

#[derive(serde::Serialize)]
struct Review {
    line: usize,
    comment: String,
    severity: String,
}
```

## Best Practices

### Tool Design
1. **Clear descriptions** - Agents rely on descriptions to understand tool purpose
2. **Validate parameters** - Return helpful error messages for invalid inputs
3. **Idempotent operations** - Tools may be retried on failure
4. **Return structured data** - JSON is preferred for agent consumption

### Hook Design
1. **Keep hooks fast** - Strict timeout enforcement (50-100ms)
2. **Don't block** - Async operations should be avoided
3. **Graceful degradation** - Return `pass_through()` on error
4. **Minimal modification** - Only modify input when necessary

### Security
1. **Request minimal permissions** - Only what you need
2. **Validate all inputs** - Tools receive arbitrary agent-generated data
3. **Sanitize outputs** - Hooks can modify agent context
4. **Log sensitive operations** - For auditability
