# Tapp Development Guide

This comprehensive guide covers everything you need to know about building Tapp extensions.

## Project Structure

```
my-app/
├── Cargo.toml           # Rust dependencies
├── src/
│   └── lib.rs           # App implementation
├── manifest.json        # App metadata & permissions
└── assets/
    └── icon.svg         # App icon (optional)
```

## Cargo.toml Configuration

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tapp = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
strip = true         # Strip debug symbols
```

## The App Trait

Every Tapp must implement the `App` trait:

```rust
use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyApp {
    // Your state fields
}

impl App for MyApp {
    /// Called once when the app is loaded
    fn init(&mut self, ctx: &Context) -> Result<()> {
        // Initialize state, load data, etc.
        Ok(())
    }

    /// Called when the app is unloaded
    fn shutdown(&mut self) -> Result<()> {
        // Cleanup, save state, etc.
        Ok(())
    }

    /// Handle UI events and actions
    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name() {
            "my_action" => {
                // Handle action
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    /// Return the current UI state
    fn render(&self) -> UITree {
        ui::text("Hello, World!")
    }
}
```

## Actions and Responses

### Action Object
```rust
pub struct Action {
    name: String,
    data: HashMap<String, Value>,
}

impl Action {
    // Get the action name
    fn name(&self) -> &str;
    
    // Get a typed parameter (returns error if missing/invalid)
    fn get<T: Deserialize>(&self, key: &str) -> Result<T>;
    
    // Get an optional parameter
    fn get_optional<T: Deserialize>(&self, key: &str) -> Option<T>;
}
```

### Response Types
```rust
// Success, no UI update needed
Response::ok()

// Success, trigger re-render
Response::render()

// Error response
Response::error("Something went wrong")

// Action not found
Response::not_found()

// With additional data
Response::ok().with_data(json!({ "result": 42 }))
```

## State Management

### Session Storage (In-Memory)
```rust
impl App for MyApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        self.storage.session_set("key", "value");
        let value: Option<String> = self.storage.session_get("key");
        Ok(())
    }
}
```

### Persistent Storage (JSON)
```rust
// Save to ~/.tyck/apps/{app-id}/data/store.json
self.storage.json_set("config", &my_config)?;
let config: Option<MyConfig> = self.storage.json_get("config")?;
```

### SQLite Database
```rust
// Uses ~/.tyck/apps/{app-id}/data/app.db
self.storage.sql_execute(
    "CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY, name TEXT)",
    &[]
)?;

self.storage.sql_execute(
    "INSERT INTO items (name) VALUES (?)",
    &[json!("Item 1")]
)?;

let rows = self.storage.sql_query(
    "SELECT * FROM items WHERE name LIKE ?",
    &[json!("%Item%")]
)?;
```

## UI Development

### Declarative UI
The `render()` method returns a declarative UI tree. Every time state changes, return a new tree.

```rust
fn render(&self) -> UITree {
    ui::panel("My Panel").children([
        ui::text("Header"),
        ui::vstack([
            ui::button("Action 1").on_click("action1"),
            ui::button("Action 2").on_click("action2"),
        ]),
    ])
}
```

### Conditional Rendering
```rust
fn render(&self) -> UITree {
    if self.is_loading {
        ui::spinner()
    } else if let Some(data) = &self.data {
        ui::text(format!("Data: {}", data))
    } else {
        ui::empty("No data loaded")
    }
}
```

### Lists and Iteration
```rust
fn render(&self) -> UITree {
    ui::vstack(
        self.items.iter().map(|item| {
            ui::hstack([
                ui::text(&item.name),
                ui::button("Delete")
                    .danger()
                    .on_click("delete")
                    .with_prop("item_id", item.id),
            ])
        }).collect::<Vec<_>>()
    )
}
```

### Event Data
```rust
// In render:
ui::button("Delete")
    .on_click("delete")
    .with_prop("on_click_data", json!({ "id": item.id }))

// In handle:
fn handle(&mut self, action: Action) -> Result<Response> {
    match action.name() {
        "delete" => {
            let id: i32 = action.get("id")?;
            self.items.retain(|item| item.id != id);
            Ok(Response::render())
        }
        _ => Ok(Response::not_found())
    }
}
```

## Drag and Drop

```rust
fn render(&self) -> UITree {
    ui::hstack([
        // Draggable item
        ui::view([ui::text("Drag me")])
            .draggable(true)
            .on_drag_start("drag_start", json!({ "item_id": 1 })),
        
        // Drop target
        ui::view([ui::text("Drop here")])
            .on_drop("drop", json!({ "target": "zone1" })),
    ])
}

fn handle(&mut self, action: Action) -> Result<Response> {
    match action.name() {
        "drag_start" => {
            let item_id: i32 = action.get("item_id")?;
            self.dragging = Some(item_id);
            Ok(Response::render())
        }
        "drop" => {
            let target: String = action.get("target")?;
            if let Some(item_id) = self.dragging.take() {
                // Move item to target
            }
            Ok(Response::render())
        }
        _ => Ok(Response::not_found())
    }
}
```

## Advanced UI Components

### Virtual List (Large Data)
```rust
fn render(&self) -> UITree {
    ui::virtual_list(
        self.items.iter().enumerate().map(|(i, item)| {
            ui::VirtualListItem {
                id: format!("item-{}", i),
                content: ui::text(&item.name),
            }
        }).collect::<Vec<_>>(),
        32 // item height in pixels
    )
    .overscan(5) // render 5 extra items above/below viewport
    .build()
}
```

### Data Grid
```rust
fn render(&self) -> UITree {
    let columns = vec![
        ui::DataGridColumn::new("name", "Name").width(200),
        ui::DataGridColumn::new("size", "Size").width(100),
        ui::DataGridColumn::new("modified", "Modified").width(150),
    ];

    let rows: Vec<ui::DataGridRow> = self.files.iter().map(|file| {
        ui::DataGridRow::new(&file.id)
            .cell("name", &file.name)
            .cell("size", file.size)
            .cell("modified", &file.modified)
    }).collect();

    ui::data_grid(columns)
        .rows(rows)
        .selectable()
        .striped()
        .on_sort("sort_changed")
        .on_row_click("row_clicked")
        .build()
}
```

### Tree View
```rust
fn render(&self) -> UITree {
    let nodes = self.build_tree_nodes();
    
    ui::tree(nodes)
        .on_select("node_selected")
        .on_toggle("node_toggled")
        .build()
}

fn build_tree_nodes(&self) -> Vec<ui::TreeNode> {
    vec![
        ui::TreeNode::new("folder1", "Documents")
            .icon("folder")
            .expanded(true)
            .children(vec![
                ui::TreeNode::new("file1", "report.pdf").icon("file"),
                ui::TreeNode::new("file2", "notes.txt").icon("file"),
            ]),
    ]
}
```

## Error Handling

### Result Type
```rust
use tapp::prelude::*;

fn load_data(&self) -> Result<Data> {
    let raw = self.storage.json_get::<String>("data")?
        .ok_or(Error::NotFound("Data not found".into()))?;
    
    serde_json::from_str(&raw)
        .map_err(|e| Error::Parse(e.to_string()))
}
```

### Error Display
```rust
fn handle(&mut self, action: Action) -> Result<Response> {
    match self.risky_operation() {
        Ok(result) => {
            self.result = Some(result);
            Ok(Response::render())
        }
        Err(e) => {
            self.error_message = Some(e.to_string());
            Ok(Response::render())
        }
    }
}

fn render(&self) -> UITree {
    ui::vstack([
        if let Some(error) = &self.error_message {
            ui::alert(error).error().into()
        } else {
            ui::empty("").into()
        },
        // ... rest of UI
    ])
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut app = MyCounter::default();
        let ctx = Context::new();
        app.init(&ctx).unwrap();
        
        let action = Action::new("increment");
        let response = app.handle(action).unwrap();
        
        assert!(response.render);
        assert_eq!(app.count, 1);
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        let mut app = MyApp::default();
        let ctx = Context::new();
        app.init(&ctx).unwrap();
        
        // Simulate user interactions
        app.handle(Action::new("add_item").with_data("name", "Test")).unwrap();
        app.handle(Action::new("add_item").with_data("name", "Test 2")).unwrap();
        
        assert_eq!(app.items.len(), 2);
        
        // Verify render output
        let ui = app.render();
        // Assert UI structure...
    }
}
```

## Hot Reload Development

During development, use `tapp dev` for automatic hot reload:

```bash
tapp dev
```

The CLI will:
1. Watch for file changes in `src/`
2. Rebuild the WASM module
3. Signal Tyck to reload the app
4. Preserve app state across reloads (when possible)

### State Serialization for Hot Reload
```rust
impl App for MyApp {
    fn serialize_state(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.state)
            .map_err(|e| Error::Serialization(e.to_string()))
    }
    
    fn deserialize_state(&mut self, state: Vec<u8>) -> Result<()> {
        self.state = serde_json::from_slice(&state)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(())
    }
}
```

## Layout Modes

Configure in `manifest.json`:

```json
{
  "ui": {
    "layout": "full"
  }
}
```

Options:
- `full` - Replace main content area
- `sidebar` - Replace context panel
- `panel` - Add below terminal
- `modal` - Overlay with backdrop

## Performance Tips

1. **Minimize render() complexity** - Cache computed values
2. **Use VirtualList** for large lists (100+ items)
3. **Debounce frequent updates** - Don't re-render on every keystroke
4. **Avoid cloning large data** - Use references where possible
5. **Profile WASM size** - Use `wasm-opt` for production builds
