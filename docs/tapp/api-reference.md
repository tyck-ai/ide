# API Reference

Complete API documentation for the `tapp` SDK.

## Core Traits

### App

The main trait that all Tapp applications must implement.

```rust
pub trait App: Default {
    /// Initialize the app. Called once when loaded.
    fn init(&mut self, ctx: &Context) -> Result<()>;
    
    /// Cleanup on shutdown.
    fn shutdown(&mut self) -> Result<()>;
    
    /// Handle UI events and actions.
    fn handle(&mut self, action: Action) -> Result<Response>;
    
    /// Return the current UI state.
    fn render(&self) -> UITree;
    
    /// Optional: Serialize state for hot reload.
    fn serialize_state(&self) -> Result<Vec<u8>> {
        Ok(vec![])
    }
    
    /// Optional: Restore state after hot reload.
    fn deserialize_state(&mut self, _state: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
```

## Types

### Context

Initialization context passed to `init()`.

```rust
pub struct Context {
    /// App ID from manifest
    pub app_id: String,
    
    /// Version from manifest
    pub version: String,
    
    /// Data directory path
    pub data_dir: PathBuf,
}

impl Context {
    /// Get an environment variable (if permitted)
    pub fn env(&self, key: &str) -> Option<String>;
    
    /// Get the workspace root path
    pub fn workspace_root(&self) -> &Path;
}
```

### Action

Represents a UI event or action.

```rust
pub struct Action {
    name: String,
    data: HashMap<String, Value>,
}

impl Action {
    /// Create a new action (testing)
    pub fn new(name: &str) -> Self;
    
    /// Create with data (testing)
    pub fn with_data<V: Serialize>(self, key: &str, value: V) -> Self;
    
    /// Get action name
    pub fn name(&self) -> &str;
    
    /// Get typed parameter
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T>;
    
    /// Get optional parameter
    pub fn get_optional<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    
    /// Check if parameter exists
    pub fn has(&self, key: &str) -> bool;
    
    /// Get raw JSON value
    pub fn raw(&self, key: &str) -> Option<&Value>;
}
```

### Response

Response to an action.

```rust
pub struct Response {
    pub success: bool,
    pub render: bool,
    pub error: Option<String>,
    pub data: Option<Value>,
}

impl Response {
    /// Success, no render needed
    pub fn ok() -> Self;
    
    /// Success, trigger re-render
    pub fn render() -> Self;
    
    /// Error response
    pub fn error(msg: &str) -> Self;
    
    /// Action not found
    pub fn not_found() -> Self;
    
    /// Add data to response
    pub fn with_data<V: Serialize>(self, data: V) -> Self;
}
```

### Result and Error

```rust
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("{0}")]
    Custom(String),
}
```

## Storage API

### Storage

Main storage interface.

```rust
pub struct Storage {
    // internal fields
}

impl Storage {
    // Session Storage (in-memory)
    
    /// Set a session value
    pub fn session_set<V: Serialize>(&self, key: &str, value: V);
    
    /// Get a session value
    pub fn session_get<V: DeserializeOwned>(&self, key: &str) -> Option<V>;
    
    /// Remove a session value
    pub fn session_remove(&self, key: &str);
    
    /// Clear all session data
    pub fn session_clear(&self);
    
    // JSON Storage (persistent)
    
    /// Set a JSON value
    pub fn json_set<V: Serialize>(&self, key: &str, value: &V) -> Result<()>;
    
    /// Get a JSON value
    pub fn json_get<V: DeserializeOwned>(&self, key: &str) -> Result<Option<V>>;
    
    /// Remove a JSON value
    pub fn json_remove(&self, key: &str) -> Result<()>;
    
    /// List all JSON keys
    pub fn json_keys(&self) -> Result<Vec<String>>;
    
    // SQLite Storage
    
    /// Execute SQL (INSERT, UPDATE, DELETE)
    pub fn sql_execute(&self, sql: &str, params: &[Value]) -> Result<usize>;
    
    /// Query SQL (SELECT)
    pub fn sql_query(&self, sql: &str, params: &[Value]) -> Result<Vec<HashMap<String, Value>>>;
}
```

## Agent API

### ToolResult

Result returned from tool functions.

```rust
pub enum ToolResult {
    Success { data: Option<Value> },
    Error { message: String },
}

impl ToolResult {
    /// Success with no data
    pub fn ok() -> Self;
    
    /// Success with JSON data
    pub fn json<V: Serialize>(data: V) -> Self;
    
    /// Success with text
    pub fn text(text: &str) -> Self;
    
    /// Error with message
    pub fn error(msg: &str) -> Self;
}
```

### HookResult

Result returned from hook functions.

```rust
pub enum HookResult {
    PassThrough,
    ModifyInput(String),
    Cancel,
}

impl HookResult {
    /// Continue without modification
    pub fn pass_through() -> Self;
    
    /// Modify the input text
    pub fn modify_input(text: String) -> Self;
    
    /// Cancel the operation
    pub fn cancel() -> Self;
}
```

### AgentSession

Interface for agent sessions.

```rust
pub struct AgentSession {
    id: String,
}

impl AgentSession {
    /// Inject text into active session
    pub fn inject(text: &str) -> Result<()>;
    
    /// Spawn a new session
    pub fn spawn(options: SpawnOptions) -> Result<Self>;
    
    /// Get session ID
    pub fn id(&self) -> &str;
    
    /// Send text to session
    pub fn send(&self, text: &str) -> Result<()>;
    
    /// Get output (non-blocking)
    pub fn get_output(&self) -> Result<Option<String>>;
    
    /// Kill the session
    pub fn kill(&self) -> Result<()>;
}

pub struct SpawnOptions {
    pub provider: Option<String>,
    pub system_prompt: Option<String>,
    pub cwd: Option<String>,
    pub visible: bool,
}
```

## UI Module

### UITree / UINode

```rust
/// Root UI structure
pub type UITree = UINode;

/// A single UI node
pub struct UINode {
    pub kind: String,
    pub props: HashMap<String, Value>,
    pub children: Vec<UINode>,
}
```

### UI Functions

All functions in the `ui` module:

```rust
// Layout
pub fn view(children: impl Into<Vec<UINode>>) -> ViewBuilder;
pub fn vstack(children: impl Into<Vec<UINode>>) -> UINode;
pub fn hstack(children: impl Into<Vec<UINode>>) -> UINode;
pub fn panel(title: &str) -> PanelBuilder;
pub fn split(direction: SplitDirection) -> SplitBuilder;
pub fn tabs(items: Vec<TabItem>) -> TabsBuilder;
pub fn modal(title: &str) -> ModalBuilder;
pub fn drawer(position: DrawerPosition) -> DrawerBuilder;
pub fn scroll(content: UINode) -> ScrollBuilder;

// Content
pub fn text(content: &str) -> TextBuilder;
pub fn code(content: &str) -> CodeBuilder;
pub fn markdown(content: &str) -> UINode;
pub fn icon(name: &str) -> UINode;
pub fn image(src: &str) -> ImageBuilder;
pub fn badge(text: &str) -> UINode;
pub fn avatar(src: &str) -> AvatarBuilder;
pub fn avatar_initials(name: &str) -> AvatarBuilder;

// Input
pub fn button(label: &str) -> ButtonBuilder;
pub fn input() -> InputBuilder;
pub fn textarea() -> TextAreaBuilder;
pub fn select(options: Vec<SelectOption>) -> SelectBuilder;
pub fn checkbox(checked: bool) -> CheckboxBuilder;
pub fn toggle(checked: bool) -> ToggleBuilder;
pub fn slider(value: f64) -> SliderBuilder;

// Data
pub fn list<I, F>(items: I, render_fn: F) -> UINode;
pub fn virtual_list(items: Vec<VirtualListItem>, item_height: u32) -> VirtualListBuilder;
pub fn table(headers: Vec<&str>) -> TableBuilder;
pub fn data_grid(columns: Vec<DataGridColumn>) -> DataGridBuilder;
pub fn tree(nodes: Vec<TreeNode>) -> TreeBuilder;

// Feedback
pub fn spinner() -> UINode;
pub fn progress(value: f64) -> UINode;
pub fn empty(message: &str) -> UINode;
pub fn alert(message: &str) -> AlertBuilder;
pub fn toast(message: &str) -> ToastBuilder;
pub fn skeleton() -> SkeletonBuilder;
```

### Builder Pattern

All components use builders:

```rust
pub struct ButtonBuilder { /* ... */ }

impl ButtonBuilder {
    pub fn primary(self) -> Self;
    pub fn danger(self) -> Self;
    pub fn disabled(self) -> Self;
    pub fn on_click(self, action: &str) -> Self;
    pub fn with_prop<V: Serialize>(self, key: &str, value: V) -> Self;
    pub fn build(self) -> UINode;
}

// Into<UINode> is implemented for automatic conversion
impl From<ButtonBuilder> for UINode { /* ... */ }
```

## Prelude

The prelude re-exports common items:

```rust
pub use crate::{
    App,
    Action,
    Response,
    Result,
    Error,
    Context,
    Storage,
    ui,
    ToolResult,
    HookResult,
    AgentSession,
    SpawnOptions,
};
```

## Attributes

### `#[tapp::app]`

```rust
// Basic usage
#[tapp::app]
#[derive(Default)]
pub struct MyApp { }

// With custom name
#[tapp::app(name = "custom_name")]
pub struct MyApp { }
```

### `#[tapp::tools]`

```rust
#[tapp::tools]
impl MyApp {
    #[tool(description = "Tool description")]
    fn tool_name(&self, args: Value) -> ToolResult { }
    
    #[tool(name = "custom_name", description = "Description")]
    fn another_tool(&mut self, args: Value) -> ToolResult { }
}
```

### `#[tapp::hooks]`

```rust
#[tapp::hooks]
impl MyApp {
    #[hook(on_before_input)]
    fn hook_name(&self, data: &Value) -> HookResult { }
    
    #[hook(on_after_output)]
    fn another_hook(&self, data: &Value) -> HookResult { }
}
```

## Constants

```rust
/// Maximum tool execution time
pub const TOOL_TIMEOUT_MS: u64 = 30_000;

/// Maximum hook execution time
pub const HOOK_TIMEOUT_MS: u64 = 100;

/// Maximum WASM memory
pub const MAX_WASM_MEMORY_PAGES: u32 = 1024; // 64MB
```
