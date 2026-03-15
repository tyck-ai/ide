export function TappQuickStartPage() {
  return (
    <div>
      <h1>Tapp Quick Start</h1>
      <p>
        Tapp is Tyck's extension framework for building custom apps, tools, and panels.
        This guide will help you create your first Tapp in under 5 minutes.
      </p>

      <blockquote>
        <p>
          <strong>What is Tapp?</strong> Tapp extensions are WASM-based apps that run inside Tyck.
          They can provide custom UI panels, integrate with AI agents, and extend the editor's functionality.
        </p>
      </blockquote>

      <h2>Prerequisites</h2>
      <p>Before you begin, make sure you have:</p>
      <ul>
        <li><a href="https://www.rust-lang.org/tools/install">Rust</a> (latest stable)</li>
        <li>WASM target: <code>rustup target add wasm32-wasip2</code></li>
      </ul>

      <h2>1. Install the Tapp CLI</h2>
      <pre><code>{`# From crates.io
cargo install tapp-cli

# Or from Homebrew (macOS/Linux)
brew tap tyck-dev/tap
brew install tapp

# Or from Scoop (Windows)
scoop bucket add tyck https://github.com/tyck-dev/scoop-bucket
scoop install tapp`}</code></pre>

      <p>Verify the installation:</p>
      <pre><code>{`tapp --version
# tapp 0.1.0`}</code></pre>

      <h2>2. Create a New Project</h2>
      <pre><code>{`tapp init my-counter
cd my-counter`}</code></pre>

      <p>This creates:</p>
      <pre><code>{`my-counter/
├── Cargo.toml
├── src/
│   └── lib.rs
├── manifest.json
└── assets/
    └── icon.svg`}</code></pre>

      <h2>3. Explore the Generated Code</h2>
      <h3>manifest.json</h3>
      <pre><code>{`{
  "id": "my-counter",
  "name": "My Counter",
  "version": "0.1.0",
  "description": "A simple counter app",
  "author": "Your Name",
  "permissions": ["storage:session"]
}`}</code></pre>

      <h3>src/lib.rs</h3>
      <pre><code>{`use tapp::prelude::*;

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
}`}</code></pre>

      <h2>4. Run in Development Mode</h2>
      <pre><code>tapp dev</code></pre>

      <p>This will:</p>
      <ol>
        <li>Build the WASM module</li>
        <li>Watch for file changes</li>
        <li>Hot reload on save</li>
      </ol>

      <p>Open Tyck IDE and press <code>Cmd+Shift+A</code> to open the app launcher.</p>

      <h2>5. Build for Release</h2>
      <pre><code>{`tapp build

# Output:
# {
#   "success": true,
#   "app_id": "my-counter",
#   "wasm_path": "./target/wasm32-wasip2/release/my_counter.wasm",
#   "wasm_size_bytes": 45632,
#   "build_time_ms": 1200
# }`}</code></pre>

      <h2>6. Install to Tyck</h2>
      <pre><code>tapp install ./manifest.json</code></pre>

      <h2>Understanding the Code</h2>
      <h3>The App Trait</h3>
      <p>Every Tapp implements the <code>App</code> trait with these methods:</p>
      <table>
        <thead>
          <tr>
            <th>Method</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr><td><code>init()</code></td><td>Called once when the app loads</td></tr>
          <tr><td><code>shutdown()</code></td><td>Called when the app unloads</td></tr>
          <tr><td><code>handle()</code></td><td>Process user actions and events</td></tr>
          <tr><td><code>render()</code></td><td>Return the current UI state</td></tr>
        </tbody>
      </table>

      <h3>The Action/Response Pattern</h3>
      <p>
        Tapp uses a unidirectional data flow. UI events trigger <code>Action</code>s, which are processed
        in <code>handle()</code>. Return <code>Response::render()</code> to update the UI.
      </p>

      <h3>Declarative UI</h3>
      <p>
        The <code>render()</code> method returns a declarative UI tree. Every time state changes,
        you return a new tree — Tyck efficiently diffs and updates the UI.
      </p>

      <h2>Adding Features</h2>
      <h3>State Persistence</h3>
      <pre><code>{`impl App for MyCounter {
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
}`}</code></pre>

      <h3>AI Agent Tools</h3>
      <pre><code>{`#[tapp::tools]
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
}`}</code></pre>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/tapp/development">Development Guide</a> — Deep dive into Tapp development</li>
        <li><a href="/docs/tapp/ui-components">UI Components</a> — Browse all available components</li>
      </ul>
    </div>
  );
}
