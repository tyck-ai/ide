# tapp

Rust SDK for building Tapp extensions for Tyck IDE.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tapp = "0.1"
```

## Quick Start

```rust
use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyApp {
    count: i32,
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
                self.count += 1;
                Ok(Response::render())
            }
            _ => Ok(Response::not_found())
        }
    }

    fn render(&self) -> UITree {
        ui::vstack([
            ui::text(format!("Count: {}", self.count)),
            ui::button("+").on_click("increment"),
        ])
    }
}
```

## Features

- **Declarative UI** - Build UIs with a React-like component model
- **Agent Integration** - Expose tools and hooks for AI agents
- **Storage APIs** - Session, JSON, and SQLite storage
- **Type-Safe** - Full Rust type safety with compile-time checks

## Documentation

- [Quick Start Guide](https://docs.tyck.dev/tapp/quick-start)
- [Development Guide](https://docs.tyck.dev/tapp/development)
- [UI Components](https://docs.tyck.dev/tapp/ui-components)
- [Agent Integration](https://docs.tyck.dev/tapp/agent-integration)
- [API Reference](https://docs.rs/tapp)

## Adding Agent Tools

```rust
#[tapp::tools]
impl MyApp {
    #[tool(description = "Get the current count")]
    fn get_count(&self, _args: serde_json::Value) -> ToolResult {
        ToolResult::json(serde_json::json!({ "count": self.count }))
    }
}
```

## License

MIT
