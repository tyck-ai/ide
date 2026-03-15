//! Database Explorer - A Tapp example app
//!
//! This app demonstrates:
//! - Agent tools for database queries
//! - Agent hooks for context enrichment
//! - Complex UI with panels and data grids
//! - State persistence

use tapp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Connection {
    id: String,
    name: String,
    db_type: String,
    host: String,
    connected: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    execution_time_ms: u64,
}

#[tapp::app]
pub struct DatabaseExplorer {
    connections: Vec<Connection>,
    active_connection: Option<String>,
    query_input: String,
    last_result: Option<QueryResult>,
    error: Option<String>,
    query_history: Vec<String>,
}

impl Default for DatabaseExplorer {
    fn default() -> Self {
        Self {
            connections: vec![
                Connection {
                    id: "demo".to_string(),
                    name: "Demo Database".to_string(),
                    db_type: "SQLite".to_string(),
                    host: "localhost".to_string(),
                    connected: true,
                },
            ],
            active_connection: Some("demo".to_string()),
            query_input: String::new(),
            last_result: None,
            error: None,
            query_history: Vec::new(),
        }
    }
}

impl App for DatabaseExplorer {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "select_connection" => {
                if let Some(id) = action.get_string("id") {
                    self.active_connection = Some(id);
                    self.error = None;
                }
                Ok(Response::render())
            }
            "update_query" => {
                if let Some(value) = action.get_string("value") {
                    self.query_input = value;
                }
                Ok(Response::ok())
            }
            "execute_query" => {
                if self.active_connection.is_none() {
                    self.error = Some("No connection selected".to_string());
                    return Ok(Response::render());
                }

                if self.query_input.trim().is_empty() {
                    self.error = Some("Query cannot be empty".to_string());
                    return Ok(Response::render());
                }

                match self.execute_query_internal(&self.query_input.clone()) {
                    Ok(result) => {
                        self.query_history.insert(0, self.query_input.clone());
                        if self.query_history.len() > 20 {
                            self.query_history.truncate(20);
                        }
                        self.last_result = Some(result);
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                        self.last_result = None;
                    }
                }
                Ok(Response::render())
            }
            "clear_results" => {
                self.last_result = None;
                self.error = None;
                self.query_input.clear();
                Ok(Response::render())
            }
            "use_history" => {
                if let Some(idx) = action.get_i64("index") {
                    if let Some(query) = self.query_history.get(idx as usize) {
                        self.query_input = query.clone();
                    }
                }
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::panel("Database Explorer").children([
            self.render_connection_bar(),
            self.render_query_section(),
            self.render_error(),
            self.render_results(),
        ])
    }
}

impl DatabaseExplorer {
    fn render_connection_bar(&self) -> UINode {
        ui::hstack(
            self.connections.iter().map(|conn| {
                let is_active = self.active_connection.as_ref() == Some(&conn.id);
                let label = format!(
                    "{} {}",
                    if conn.connected { "●" } else { "○" },
                    conn.name
                );
                
                let btn = if is_active {
                    ui::button(&label).primary()
                } else {
                    ui::button(&label)
                };
                
                btn.on_click("select_connection")
                    .with_prop("on_click_data", serde_json::json!({ "id": &conn.id }))
            }).collect::<Vec<_>>()
        )
    }

    fn render_query_section(&self) -> UINode {
        ui::vstack([
            ui::text("SQL Query").with_class("font-bold"),
            ui::textarea()
                .value(&self.query_input)
                .placeholder("SELECT * FROM users WHERE ...")
                .rows(4)
                .build()
                .on_change("update_query"),
            ui::hstack([
                ui::button("Execute")
                    .primary()
                    .on_click("execute_query"),
                ui::button("Clear")
                    .on_click("clear_results"),
            ]),
            self.render_history(),
        ])
    }

    fn render_history(&self) -> UINode {
        if self.query_history.is_empty() {
            return ui::empty("");
        }

        ui::vstack([
            ui::text("Recent Queries").with_class("text-sm opacity-70"),
            ui::vstack(
                self.query_history.iter().take(5).enumerate().map(|(i, q)| {
                    let display = if q.len() > 50 {
                        format!("{}...", &q[..50])
                    } else {
                        q.clone()
                    };
                    ui::button(&display)
                        .on_click("use_history")
                        .with_class("text-left text-sm")
                        .with_prop("on_click_data", serde_json::json!({ "index": i }))
                }).collect::<Vec<_>>()
            ),
        ])
    }

    fn render_error(&self) -> UINode {
        if let Some(error) = &self.error {
            ui::vstack([
                ui::text(&format!("Error: {}", error))
                    .with_class("text-red-500"),
            ])
        } else {
            ui::empty("")
        }
    }

    fn render_results(&self) -> UINode {
        if let Some(result) = &self.last_result {
            let headers: Vec<String> = result.columns.clone();
            let mut table = ui::table(headers);
            
            for row in &result.rows {
                let cells: Vec<UINode> = row.iter()
                    .map(|cell| ui::text(&format!("{}", cell)))
                    .collect();
                table = table.row(cells);
            }
            
            ui::vstack([
                ui::text(&format!(
                    "Results: {} rows ({} ms)",
                    result.rows.len(),
                    result.execution_time_ms
                )).with_class("text-sm opacity-70"),
                table.bordered().build(),
            ])
        } else {
            ui::empty("Execute a query to see results")
        }
    }

    fn execute_query_internal(&self, sql: &str) -> std::result::Result<QueryResult, String> {
        let sql_lower = sql.to_lowercase();
        
        if sql_lower.contains("users") {
            Ok(QueryResult {
                columns: vec!["id".into(), "name".into(), "email".into(), "created_at".into()],
                rows: vec![
                    vec![
                        serde_json::json!(1),
                        serde_json::json!("Alice"),
                        serde_json::json!("alice@example.com"),
                        serde_json::json!("2024-01-15"),
                    ],
                    vec![
                        serde_json::json!(2),
                        serde_json::json!("Bob"),
                        serde_json::json!("bob@example.com"),
                        serde_json::json!("2024-02-20"),
                    ],
                    vec![
                        serde_json::json!(3),
                        serde_json::json!("Charlie"),
                        serde_json::json!("charlie@example.com"),
                        serde_json::json!("2024-03-10"),
                    ],
                ],
                execution_time_ms: 12,
            })
        } else if sql_lower.contains("orders") {
            Ok(QueryResult {
                columns: vec!["id".into(), "user_id".into(), "total".into(), "status".into()],
                rows: vec![
                    vec![
                        serde_json::json!(101),
                        serde_json::json!(1),
                        serde_json::json!(99.99),
                        serde_json::json!("completed"),
                    ],
                    vec![
                        serde_json::json!(102),
                        serde_json::json!(2),
                        serde_json::json!(149.50),
                        serde_json::json!("pending"),
                    ],
                ],
                execution_time_ms: 8,
            })
        } else if sql_lower.starts_with("select") {
            Ok(QueryResult {
                columns: vec!["result".into()],
                rows: vec![vec![serde_json::json!("Query executed successfully")]],
                execution_time_ms: 5,
            })
        } else {
            Err(format!("Unsupported query: {}", sql))
        }
    }
}

// Agent tools for AI integration
#[tapp::tools]
impl DatabaseExplorer {
    /// Execute a SQL query against the connected database
    #[tool(name = "query_database", description = "Execute a SQL query and return results. Supports SELECT queries on tables like 'users' and 'orders'.")]
    fn query_database(&self, args: serde_json::Value) -> ToolResult {
        let sql = match args.get("sql").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::error("Missing 'sql' parameter"),
        };

        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        if self.active_connection.is_none() {
            return ToolResult::error("No database connection active");
        }

        match self.execute_query_internal(sql) {
            Ok(mut result) => {
                if result.rows.len() > limit {
                    result.rows.truncate(limit);
                }
                ToolResult::json(serde_json::json!({
                    "columns": result.columns,
                    "rows": result.rows,
                    "row_count": result.rows.len(),
                    "execution_time_ms": result.execution_time_ms
                }))
            }
            Err(e) => ToolResult::error(&e)
        }
    }

    /// List all tables in the database
    #[tool(name = "list_tables", description = "List all available tables in the connected database")]
    fn list_tables(&self, _args: serde_json::Value) -> ToolResult {
        if self.active_connection.is_none() {
            return ToolResult::error("No database connection active");
        }

        ToolResult::json(serde_json::json!({
            "tables": ["users", "orders", "products", "categories"],
            "connection": self.active_connection
        }))
    }

    /// Get schema for a table
    #[tool(name = "describe_table", description = "Get the column definitions for a specific table")]
    fn describe_table(&self, args: serde_json::Value) -> ToolResult {
        let table = match args.get("table").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return ToolResult::error("Missing 'table' parameter"),
        };

        if self.active_connection.is_none() {
            return ToolResult::error("No database connection active");
        }

        let schema = match table {
            "users" => serde_json::json!({
                "table": "users",
                "columns": [
                    { "name": "id", "type": "INTEGER", "primary_key": true },
                    { "name": "name", "type": "VARCHAR(255)" },
                    { "name": "email", "type": "VARCHAR(255)" },
                    { "name": "created_at", "type": "TIMESTAMP" }
                ]
            }),
            "orders" => serde_json::json!({
                "table": "orders",
                "columns": [
                    { "name": "id", "type": "INTEGER", "primary_key": true },
                    { "name": "user_id", "type": "INTEGER", "foreign_key": "users.id" },
                    { "name": "total", "type": "DECIMAL(10,2)" },
                    { "name": "status", "type": "VARCHAR(50)" }
                ]
            }),
            _ => {
                return ToolResult::error(&format!("Unknown table: {}", table));
            }
        };

        ToolResult::json(schema)
    }

    /// Get current connection status
    #[tool(name = "connection_status", description = "Get the current database connection status")]
    fn connection_status(&self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({
            "connections": self.connections.iter().map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "type": c.db_type,
                    "host": c.host,
                    "connected": c.connected
                })
            }).collect::<Vec<_>>(),
            "active": self.active_connection
        }))
    }
}

// Agent hooks for context enrichment
#[tapp::hooks]
impl DatabaseExplorer {
    /// Add database context when the user asks about SQL/databases
    #[hook(on_before_input)]
    fn enrich_with_db_context(&self, data: &serde_json::Value) -> HookResult {
        let input = data.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let keywords = ["database", "sql", "query", "table", "select", "insert", "update", "delete"];
        let is_db_related = keywords.iter().any(|kw| input.to_lowercase().contains(kw));

        if is_db_related && self.active_connection.is_some() {
            let context = format!(
                "{}\n\n---\n**Database Context:**\n- Connected to: {} ({})\n- Available tables: users, orders, products, categories\n- Recent queries: {}",
                input,
                self.connections.first().map(|c| c.name.as_str()).unwrap_or("unknown"),
                self.active_connection.as_ref().unwrap_or(&"none".to_string()),
                self.query_history.len()
            );
            HookResult::modify_input(context)
        } else {
            HookResult::pass_through()
        }
    }
}
