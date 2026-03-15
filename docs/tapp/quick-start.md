# Quick Start: Build Your First Tapp

This guide will help you create a simple counter app in under 5 minutes.

## 1. Create a New Project

```bash
tapp init my-counter
cd my-counter
```

This creates:
```
my-counter/
├── Cargo.toml
├── src/
│   └── lib.rs
├── manifest.json
└── assets/
    └── icon.svg
```

## 2. Explore the Generated Code

### manifest.json
```json
{
  "id": "my-counter",
  "name": "My Counter",
  "version": "0.1.0",
  "description": "A simple counter app",
  "author": "Your Name",
  "permissions": ["storage:session"]
}
```

### src/lib.rs
```rust
use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyCounter {
    count: i32,
}

impl App for MyCounter {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "increment" => {
                self.count += 1;
                Ok(Response::render())
            }
            "decrement" => {
                self.count -= 1;
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::vstack([
            ui::text(format!("Count: {}", self.count))
                .with_class("text-2xl font-bold"),
            ui::hstack([
                ui::button("−").on_click("decrement"),
                ui::button("+").on_click("increment"),
            ]),
        ])
    }
}
```

## 3. Run in Development Mode

```bash
tapp dev
```

This will:
1. Build the WASM module
2. Watch for file changes
3. Hot reload on save

Open Tyck IDE and press `Cmd+Shift+A` to open the app launcher.

## 4. Build for Release

```bash
tapp build
```

Output:
```json
{
  "success": true,
  "app_id": "my-counter",
  "wasm_path": "./target/wasm32-wasip2/release/my_counter.wasm",
  "wasm_size_bytes": 45632,
  "build_time_ms": 1200
}
```

## 5. Install to Tyck

```bash
tapp install ./manifest.json
```

## Next Steps

- [UI Components Reference](./ui-components.md) - Learn all available components
- [Agent Integration](./agent-integration.md) - Add AI agent tools
- [Development Guide](./development.md) - Deep dive into app development
- [Examples](./examples/) - Browse complete example apps

## Common Patterns

### Adding State Persistence

```rust
#[tapp::app]
#[derive(Default)]
pub struct MyCounter {
    count: i32,
    storage: Storage,
}

impl App for MyCounter {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        // Load persisted count
        if let Some(saved) = self.storage.session_get::<i32>("count") {
            self.count = saved;
        }
        Ok(())
    }

    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "increment" => {
                self.count += 1;
                self.storage.session_set("count", self.count);
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }
    
    // ...
}
```

### Adding an Agent Tool

```rust
#[tapp::tools]
impl MyCounter {
    #[tool(description = "Get the current count")]
    fn get_count(&self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({
            "count": self.count
        }))
    }

    #[tool(description = "Set the count to a specific value")]
    fn set_count(&mut self, args: serde_json::Value) -> ToolResult {
        if let Some(value) = args.get("value").and_then(|v| v.as_i64()) {
            self.count = value as i32;
            ToolResult::ok()
        } else {
            ToolResult::error("Missing 'value' parameter")
        }
    }
}
```

### Input Handling

```rust
#[tapp::app]
#[derive(Default)]
pub struct MyApp {
    input_value: String,
}

impl App for MyApp {
    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "update_input" => {
                self.input_value = action.get("value")?;
                Ok(Response::render())
            }
            "submit" => {
                // Process input_value
                self.input_value.clear();
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::vstack([
            ui::input()
                .value(&self.input_value)
                .placeholder("Enter text...")
                .on_change("update_input"),
            ui::button("Submit").on_click("submit"),
        ])
    }
}
```
