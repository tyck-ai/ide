//! Database Explorer Example
//! 
//! An app that exposes database tools for AI agents to query databases.

use tapp::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
struct Connection {
    id: String,
    name: String,
    database_type: String,
    host: String,
    port: u16,
    database: String,
    connected: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    affected_rows: Option<usize>,
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
    storage: Storage,
}

impl Default for DatabaseExplorer {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            active_connection: None,
            query_input: String::new(),
            last_result: None,
            error: None,
            query_history: Vec::new(),
            storage: Storage::default(),
        }
    }
}

impl App for DatabaseExplorer {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        // Load saved connections
        if let Ok(Some(conns)) = self.storage.json_get::<Vec<Connection>>("connections") {
            self.connections = conns;
        }
        if let Ok(Some(history)) = self.storage.json_get::<Vec<String>>("query_history") {
            self.query_history = history;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.storage.json_set("connections", &self.connections)?;
        self.storage.json_set("query_history", &self.query_history)?;
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "select_connection" => {
                let id: String = action.get("id")?;
                self.active_connection = Some(id);
                self.error = None;
                Ok(Response::render())
            }
            "update_query" => {
                self.query_input = action.get("value")?;
                Ok(Response::render())
            }
            "execute_query" => {
                if let Some(conn_id) = &self.active_connection {
                    match self.execute_query_internal(&self.query_input.clone()) {
                        Ok(result) => {
                            // Save to history
                            if !self.query_input.is_empty() {
                                self.query_history.insert(0, self.query_input.clone());
                                if self.query_history.len() > 50 {
                                    self.query_history.truncate(50);
                                }
                            }
                            self.last_result = Some(result);
                            self.error = None;
                        }
                        Err(e) => {
                            self.error = Some(e);
                            self.last_result = None;
                        }
                    }
                } else {
                    self.error = Some("No connection selected".to_string());
                }
                Ok(Response::render())
            }
            "clear_results" => {
                self.last_result = None;
                self.error = None;
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::split(ui::SplitDirection::Horizontal)
            .panel(self.render_sidebar(), 0.25)
            .panel(self.render_main(), 0.75)
            .build()
    }
}

impl DatabaseExplorer {
    fn render_sidebar(&self) -> UINode {
        ui::panel("Connections").children([
            ui::vstack(
                self.connections.iter().map(|conn| {
                    let is_active = self.active_connection.as_ref() == Some(&conn.id);
                    ui::hstack([
                        ui::icon(if conn.connected { "database" } else { "database-off" }),
                        ui::text(&conn.name),
                    ])
                    .with_class(if is_active { "bg-blue-100" } else { "" })
                    .on_click("select_connection")
                    .with_prop("on_click_data", serde_json::json!({ "id": &conn.id }))
                }).collect::<Vec<_>>()
            ),
        ])
    }

    fn render_main(&self) -> UINode {
        ui::vstack([
            // Query input
            ui::panel("Query").children([
                ui::textarea()
                    .value(&self.query_input)
                    .placeholder("SELECT * FROM ...")
                    .rows(5)
                    .on_change("update_query"),
                ui::hstack([
                    ui::button("Execute")
                        .primary()
                        .on_click("execute_query")
                        .disabled(self.active_connection.is_none()),
                    ui::button("Clear")
                        .on_click("clear_results"),
                ]),
            ]),

            // Error display
            if let Some(error) = &self.error {
                ui::alert(error).error().build().into()
            } else {
                ui::empty("").into()
            },

            // Results
            if let Some(result) = &self.last_result {
                self.render_results(result).into()
            } else {
                ui::empty("Execute a query to see results").into()
            },
        ])
    }

    fn render_results(&self, result: &QueryResult) -> UINode {
        let columns: Vec<ui::DataGridColumn> = result.columns.iter()
            .map(|col| ui::DataGridColumn::new(col, col).width(150))
            .collect();

        let rows: Vec<ui::DataGridRow> = result.rows.iter().enumerate()
            .map(|(i, row)| {
                let mut grid_row = ui::DataGridRow::new(&format!("row-{}", i));
                for (j, value) in row.iter().enumerate() {
                    if let Some(col_name) = result.columns.get(j) {
                        grid_row = grid_row.cell(col_name, &format!("{}", value));
                    }
                }
                grid_row
            })
            .collect();

        ui::panel(&format!("Results ({} rows, {}ms)", result.rows.len(), result.execution_time_ms))
            .children([
                ui::data_grid(columns)
                    .rows(rows)
                    .striped()
                    .build()
            ])
    }

    fn execute_query_internal(&self, _sql: &str) -> std::result::Result<QueryResult, String> {
        // Simulated - in real app, connect to actual database
        Ok(QueryResult {
            columns: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            rows: vec![
                vec![
                    serde_json::json!(1),
                    serde_json::json!("John Doe"),
                    serde_json::json!("john@example.com"),
                ],
                vec![
                    serde_json::json!(2),
                    serde_json::json!("Jane Smith"),
                    serde_json::json!("jane@example.com"),
                ],
            ],
            affected_rows: None,
            execution_time_ms: 42,
        })
    }
}

// Agent tools
#[tapp::tools]
impl DatabaseExplorer {
    #[tool(description = "Execute a SQL query against the connected database")]
    fn query_database(&self, args: serde_json::Value) -> ToolResult {
        let sql = args.get("sql")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'sql' parameter")?;

        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        if self.active_connection.is_none() {
            return ToolResult::error("No database connection active");
        }

        match self.execute_query_internal(sql) {
            Ok(mut result) => {
                // Apply limit
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

    #[tool(description = "List all tables in the connected database")]
    fn list_tables(&self, _args: serde_json::Value) -> ToolResult {
        if self.active_connection.is_none() {
            return ToolResult::error("No database connection active");
        }

        // Simulated
        ToolResult::json(serde_json::json!({
            "tables": ["users", "orders", "products", "categories"]
        }))
    }

    #[tool(description = "Get the schema for a table (columns and types)")]
    fn describe_table(&self, args: serde_json::Value) -> ToolResult {
        let table = args.get("table")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'table' parameter")?;

        if self.active_connection.is_none() {
            return ToolResult::error("No database connection active");
        }

        // Simulated
        ToolResult::json(serde_json::json!({
            "table": table,
            "columns": [
                { "name": "id", "type": "INTEGER", "nullable": false, "primary_key": true },
                { "name": "name", "type": "VARCHAR(255)", "nullable": false, "primary_key": false },
                { "name": "email", "type": "VARCHAR(255)", "nullable": true, "primary_key": false },
                { "name": "created_at", "type": "TIMESTAMP", "nullable": false, "primary_key": false }
            ]
        }))
    }

    #[tool(description = "List available database connections")]
    fn list_connections(&self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({
            "connections": self.connections.iter().map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "type": c.database_type,
                    "connected": c.connected
                })
            }).collect::<Vec<_>>(),
            "active": self.active_connection
        }))
    }

    #[tool(description = "Switch to a different database connection")]
    fn switch_connection(&mut self, args: serde_json::Value) -> ToolResult {
        let conn_id = args.get("connection_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'connection_id' parameter")?;

        if self.connections.iter().any(|c| c.id == conn_id) {
            self.active_connection = Some(conn_id.to_string());
            ToolResult::ok()
        } else {
            ToolResult::error(&format!("Connection '{}' not found", conn_id))
        }
    }
}

// Agent hooks
#[tapp::hooks]
impl DatabaseExplorer {
    /// Enrich agent input with database context when relevant
    #[hook(on_before_input)]
    fn add_db_context(&self, data: &serde_json::Value) -> HookResult {
        let input = data.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // If the user is asking about database/sql, add context
        let keywords = ["database", "sql", "query", "table", "select", "insert"];
        let is_db_related = keywords.iter().any(|kw| input.to_lowercase().contains(kw));

        if is_db_related && self.active_connection.is_some() {
            let context = format!(
                "{}\n\n---\nDatabase Context:\n- Connected to: {}\n- Available tables: users, orders, products, categories",
                input,
                self.active_connection.as_ref().unwrap_or(&"none".to_string())
            );
            HookResult::modify_input(context)
        } else {
            HookResult::pass_through()
        }
    }
}
