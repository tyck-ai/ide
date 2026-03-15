export function TappDevelopmentPage() {
  return (
    <div>
      <h1>Tapp Development Guide</h1>
      <p>
        This comprehensive guide covers everything you need to know about building Tapp extensions.
      </p>

      <h2>Project Structure</h2>
      <pre><code>{`my-app/
├── Cargo.toml           # Rust dependencies
├── src/
│   └── lib.rs           # App implementation
├── manifest.json        # App metadata & permissions
└── assets/
    └── icon.svg         # App icon (optional)`}</code></pre>

      <h2>Cargo.toml Configuration</h2>
      <pre><code>{`[package]
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
strip = true         # Strip debug symbols`}</code></pre>

      <h2>The App Trait</h2>
      <p>Every Tapp must implement the <code>App</code> trait:</p>
      <pre><code>{`use tapp::prelude::*;

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
}`}</code></pre>

      <h2>Actions and Responses</h2>
      <h3>Action Object</h3>
      <pre><code>{`pub struct Action {
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
}`}</code></pre>

      <h3>Response Types</h3>
      <pre><code>{`// Success, no UI update needed
Response::ok()

// Success, trigger re-render
Response::render()

// Error response
Response::error("Something went wrong")

// Action not found
Response::not_found()

// With additional data
Response::ok().with_data(json!({ "result": 42 }))`}</code></pre>

      <h2>State Management</h2>
      <h3>Session Storage (In-Memory)</h3>
      <pre><code>{`impl App for MyApp {
    fn init(&mut self, _ctx: &Context) -> Result<()> {
        self.storage.session_set("key", "value");
        let value: Option<String> = self.storage.session_get("key");
        Ok(())
    }
}`}</code></pre>

      <h3>Persistent Storage (JSON)</h3>
      <pre><code>{`// Save to ~/.tyck/apps/{app-id}/data/store.json
self.storage.json_set("config", &my_config)?;
let config: Option<MyConfig> = self.storage.json_get("config")?;`}</code></pre>

      <h3>SQLite Database</h3>
      <pre><code>{`// Uses ~/.tyck/apps/{app-id}/data/app.db
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
)?;`}</code></pre>

      <h2>Event Handling</h2>
      <h3>Basic Events</h3>
      <pre><code>{`// In render:
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
}`}</code></pre>

      <h3>Input Handling</h3>
      <pre><code>{`#[tapp::app]
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
}`}</code></pre>

      <h2>Drag and Drop</h2>
      <pre><code>{`fn render(&self) -> UITree {
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
}`}</code></pre>

      <h2>Manifest Reference</h2>
      <pre><code>{`{
  "id": "my-app",
  "name": "My App",
  "version": "0.1.0",
  "description": "A description of my app",
  "author": "Your Name <you@example.com>",
  "repository": "https://github.com/you/my-app",
  "homepage": "https://myapp.example.com",
  "license": "MIT",
  
  "permissions": [
    "storage:session",
    "storage:persistent", 
    "storage:sqlite",
    "network:fetch",
    "fs:read",
    "fs:write"
  ],
  
  "ui": {
    "layout": "full",
    "icon": "./assets/icon.svg"
  },
  
  "tools": [
    {
      "name": "get_count",
      "description": "Get the current count value"
    }
  ]
}`}</code></pre>

      <h3>Layout Options</h3>
      <table>
        <thead>
          <tr>
            <th>Value</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr><td><code>full</code></td><td>Replace main content area</td></tr>
          <tr><td><code>sidebar</code></td><td>Replace context panel</td></tr>
          <tr><td><code>panel</code></td><td>Add below terminal</td></tr>
          <tr><td><code>modal</code></td><td>Overlay with backdrop</td></tr>
        </tbody>
      </table>

      <h2>Error Handling</h2>
      <pre><code>{`use tapp::prelude::*;

fn load_data(&self) -> Result<Data> {
    let raw = self.storage.json_get::<String>("data")?
        .ok_or(Error::NotFound("Data not found".into()))?;
    
    serde_json::from_str(&raw)
        .map_err(|e| Error::Parse(e.to_string()))
}

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
}`}</code></pre>

      <h2>Testing</h2>
      <h3>Unit Tests</h3>
      <pre><code>{`#[cfg(test)]
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
}`}</code></pre>

      <h2>Hot Reload Development</h2>
      <p>During development, use <code>tapp dev</code> for automatic hot reload:</p>
      <pre><code>tapp dev</code></pre>

      <p>The CLI will:</p>
      <ol>
        <li>Watch for file changes in <code>src/</code></li>
        <li>Rebuild the WASM module</li>
        <li>Signal Tyck to reload the app</li>
        <li>Preserve app state across reloads (when possible)</li>
      </ol>

      <h2>Performance Tips</h2>
      <ol>
        <li><strong>Minimize render() complexity</strong> — Cache computed values</li>
        <li><strong>Use VirtualList</strong> for large lists (100+ items)</li>
        <li><strong>Debounce frequent updates</strong> — Don't re-render on every keystroke</li>
        <li><strong>Avoid cloning large data</strong> — Use references where possible</li>
        <li><strong>Profile WASM size</strong> — Use <code>wasm-opt</code> for production builds</li>
      </ol>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/tapp/ui-components">UI Components Reference</a> — All available components</li>
        <li><a href="/docs/tapp">Quick Start</a> — Create your first app</li>
      </ul>
    </div>
  );
}
