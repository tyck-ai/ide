//! Todo List Example
//! 
//! A full-featured todo list with persistence, filtering, and agent tools.

use tapp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

#[tapp::app]
pub struct TodoList {
    todos: Vec<Todo>,
    next_id: u32,
    input: String,
    filter: Filter,
    storage: Storage,
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            todos: Vec::new(),
            next_id: 1,
            input: String::new(),
            filter: Filter::All,
            storage: Storage::default(),
        }
    }
}

impl App for TodoList {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        // Load saved todos
        if let Ok(Some(saved)) = self.storage.json_get::<Vec<Todo>>("todos") {
            self.todos = saved;
            self.next_id = self.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.save_todos()
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "update_input" => {
                self.input = action.get("value")?;
                Ok(Response::render())
            }
            "add_todo" => {
                if !self.input.trim().is_empty() {
                    self.todos.push(Todo {
                        id: self.next_id,
                        text: self.input.clone(),
                        completed: false,
                    });
                    self.next_id += 1;
                    self.input.clear();
                    self.save_todos()?;
                }
                Ok(Response::render())
            }
            "toggle_todo" => {
                let id: u32 = action.get("id")?;
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.completed = !todo.completed;
                    self.save_todos()?;
                }
                Ok(Response::render())
            }
            "delete_todo" => {
                let id: u32 = action.get("id")?;
                self.todos.retain(|t| t.id != id);
                self.save_todos()?;
                Ok(Response::render())
            }
            "set_filter" => {
                let filter: String = action.get("filter")?;
                self.filter = match filter.as_str() {
                    "active" => Filter::Active,
                    "completed" => Filter::Completed,
                    _ => Filter::All,
                };
                Ok(Response::render())
            }
            "clear_completed" => {
                self.todos.retain(|t| !t.completed);
                self.save_todos()?;
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        let filtered: Vec<_> = self.todos.iter()
            .filter(|t| match self.filter {
                Filter::All => true,
                Filter::Active => !t.completed,
                Filter::Completed => t.completed,
            })
            .collect();

        let active_count = self.todos.iter().filter(|t| !t.completed).count();
        let completed_count = self.todos.iter().filter(|t| t.completed).count();

        ui::panel("Todo List").children([
            // Input section
            ui::hstack([
                ui::input()
                    .value(&self.input)
                    .placeholder("What needs to be done?")
                    .on_change("update_input")
                    .on_submit("add_todo"),
                ui::button("Add")
                    .primary()
                    .on_click("add_todo"),
            ]),

            // Filter tabs
            ui::hstack([
                self.filter_button("All", Filter::All),
                self.filter_button("Active", Filter::Active),
                self.filter_button("Completed", Filter::Completed),
            ]),

            // Todo list
            if filtered.is_empty() {
                ui::empty("No todos").into()
            } else {
                ui::vstack(
                    filtered.iter().map(|todo| self.render_todo(todo)).collect::<Vec<_>>()
                ).into()
            },

            // Footer
            ui::hstack([
                ui::text(format!("{} items left", active_count)),
                if completed_count > 0 {
                    ui::button("Clear completed")
                        .on_click("clear_completed")
                        .into()
                } else {
                    ui::empty("").into()
                },
            ]),
        ])
    }
}

impl TodoList {
    fn save_todos(&self) -> Result<()> {
        self.storage.json_set("todos", &self.todos)
    }

    fn filter_button(&self, label: &str, filter: Filter) -> UINode {
        let mut btn = ui::button(label)
            .on_click("set_filter")
            .with_prop("on_click_data", serde_json::json!({ 
                "filter": match filter {
                    Filter::All => "all",
                    Filter::Active => "active",
                    Filter::Completed => "completed",
                }
            }));
        
        if self.filter == filter {
            btn = btn.primary();
        }
        
        btn.build()
    }

    fn render_todo(&self, todo: &Todo) -> UINode {
        ui::hstack([
            ui::checkbox(todo.completed)
                .on_change("toggle_todo")
                .with_prop("on_change_data", serde_json::json!({ "id": todo.id })),
            ui::text(&todo.text)
                .with_class(if todo.completed { "line-through opacity-50" } else { "" }),
            ui::button("×")
                .danger()
                .on_click("delete_todo")
                .with_prop("on_click_data", serde_json::json!({ "id": todo.id })),
        ])
    }
}

// Agent tools
#[tapp::tools]
impl TodoList {
    #[tool(description = "Add a new todo item")]
    fn add_todo(&mut self, args: serde_json::Value) -> ToolResult {
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'text' parameter")?;

        self.todos.push(Todo {
            id: self.next_id,
            text: text.to_string(),
            completed: false,
        });
        self.next_id += 1;
        let _ = self.save_todos();
        
        ToolResult::ok()
    }

    #[tool(description = "List all todos, optionally filtered by status")]
    fn list_todos(&self, args: serde_json::Value) -> ToolResult {
        let filter = args.get("filter").and_then(|v| v.as_str());
        
        let filtered: Vec<_> = self.todos.iter()
            .filter(|t| match filter {
                Some("active") => !t.completed,
                Some("completed") => t.completed,
                _ => true,
            })
            .collect();

        ToolResult::json(serde_json::json!({
            "todos": filtered,
            "count": filtered.len()
        }))
    }

    #[tool(description = "Mark a todo as completed")]
    fn complete_todo(&mut self, args: serde_json::Value) -> ToolResult {
        let id = args.get("id")
            .and_then(|v| v.as_u64())
            .ok_or("Missing 'id' parameter")? as u32;

        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = true;
            let _ = self.save_todos();
            ToolResult::ok()
        } else {
            ToolResult::error(&format!("Todo {} not found", id))
        }
    }

    #[tool(description = "Delete a todo item")]
    fn delete_todo(&mut self, args: serde_json::Value) -> ToolResult {
        let id = args.get("id")
            .and_then(|v| v.as_u64())
            .ok_or("Missing 'id' parameter")? as u32;

        let before = self.todos.len();
        self.todos.retain(|t| t.id != id);
        
        if self.todos.len() < before {
            let _ = self.save_todos();
            ToolResult::ok()
        } else {
            ToolResult::error(&format!("Todo {} not found", id))
        }
    }
}
