pub fn manifest(id: &str, name: &str) -> String {
    let value = serde_json::json!({
        "id": id,
        "name": name,
        "version": "0.1.0",
        "description": "A Tapp extension for Tyck IDE",
        "permissions": ["storage:session"],
        "ui": { "layout": "full" }
    });
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

pub fn cargo_toml(id: &str) -> String {
    let crate_name = id.replace('-', "_");
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tapp = "0.1"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        crate_name
    )
}

pub fn lib_rs_minimal() -> String {
    r#"use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyApp {
    counter: u32,
}

impl App for MyApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "increment" => {
                self.counter += 1;
                Ok(Response::render())
            }
            "decrement" => {
                self.counter = self.counter.saturating_sub(1);
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::vstack([
            ui::text(format!("Count: {}", self.counter)),
            ui::hstack([
                ui::button("-").on_click("decrement"),
                ui::button("+").on_click("increment"),
            ]),
        ])
    }
}
"#
    .to_string()
}

pub fn lib_rs_tool() -> String {
    r#"use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyToolApp {
    last_result: Option<String>,
}

impl App for MyToolApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, _action: Action) -> Result<Response> {
        Ok(Response::ok())
    }

    fn render(&self) -> UITree {
        ui::vstack([
            ui::text("Tool App"),
            if let Some(result) = &self.last_result {
                ui::text(format!("Last result: {}", result))
            } else {
                ui::empty("No tool calls yet")
            },
        ])
    }
}

#[tapp::tools]
impl MyToolApp {
    #[tool(name = "example_tool", description = "An example tool that echoes input")]
    fn example_tool(&mut self, args: serde_json::Value) -> ToolResult {
        let input = args["input"].as_str().unwrap_or("no input");
        self.last_result = Some(input.to_string());
        ToolResult::json(serde_json::json!({
            "echo": input,
            "processed": true
        }))
    }
}
"#
    .to_string()
}

pub fn lib_rs_full() -> String {
    r#"use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyFullApp {
    messages: Vec<String>,
    input: String,
}

impl App for MyFullApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "update_input" => {
                if let Some(value) = action.get_string("value") {
                    self.input = value;
                }
                Ok(Response::render())
            }
            "send_message" => {
                if !self.input.is_empty() {
                    self.messages.push(self.input.clone());
                    self.input.clear();
                }
                Ok(Response::render())
            }
            "clear_messages" => {
                self.messages.clear();
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::panel("My Full App").children([
            ui::vstack([
                ui::list(&self.messages, |msg| {
                    ui::text(msg.clone())
                }),
                ui::hstack([
                    ui::input()
                        .value(&self.input)
                        .placeholder("Enter a message...")
                        .on_change("update_input")
                        .on_submit("send_message"),
                    ui::button("Send").primary().on_click("send_message"),
                ]),
                ui::button("Clear").danger().on_click("clear_messages"),
            ])
        ])
    }
}

#[tapp::tools]
impl MyFullApp {
    #[tool(name = "add_message", description = "Add a message to the list")]
    fn add_message(&mut self, args: serde_json::Value) -> ToolResult {
        let message = args["message"].as_str().unwrap_or("").to_string();
        if message.is_empty() {
            return ToolResult::error("Message cannot be empty");
        }
        self.messages.push(message.clone());
        ToolResult::json(serde_json::json!({
            "added": message,
            "total_messages": self.messages.len()
        }))
    }

    #[tool(name = "get_messages", description = "Get all messages")]
    fn get_messages(&mut self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({
            "messages": self.messages,
            "count": self.messages.len()
        }))
    }
}

#[tapp::hooks]
impl MyFullApp {
    #[hook(on_before_input)]
    fn enrich_input(&self, data: &serde_json::Value) -> HookResult {
        HookResult::pass_through()
    }
}
"#
    .to_string()
}

pub fn lib_rs_sidebar() -> String {
    r#"use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MySidebarApp {
    items: Vec<String>,
    selected: Option<usize>,
}

impl App for MySidebarApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        self.items = vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
        ];
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "select_item" => {
                if let Some(index) = action.get_i64("index") {
                    self.selected = Some(index as usize);
                }
                Ok(Response::render())
            }
            "add_item" => {
                let name = format!("Item {}", self.items.len() + 1);
                self.items.push(name);
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::panel("Sidebar App").children([
            ui::vstack(
                self.items.iter().enumerate().map(|(i, item)| {
                    let is_selected = self.selected == Some(i);
                    ui::button(item.as_str())
                        .with_class(if is_selected { "selected" } else { "" })
                        .on_click("select_item")
                        .with_prop("data", serde_json::json!({ "index": i }))
                }).collect::<Vec<_>>()
            ),
            ui::button("Add Item").on_click("add_item"),
        ])
    }
}
"#
    .to_string()
}

pub fn gitignore() -> String {
    r#"/target
Cargo.lock
"#
    .to_string()
}

pub fn readme(name: &str) -> String {
    format!(
        r#"# {}

A Tapp extension for Tyck IDE.

## Development

```bash
# Start development mode (watch for changes)
tapp dev

# Build for release
tapp build

# Install the app
tapp install .
```

## Usage

1. Open Tyck
2. Press `Cmd+Shift+A` to open the app launcher
3. Select this app
"#,
        name
    )
}
