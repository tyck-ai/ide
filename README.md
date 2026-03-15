# Tyck

An AI-native code editor built with Tauri, featuring a powerful extension system called **Tapp**.

> **Status:** Active development. Contributions welcome!

## Overview

Tyck is a modern code editor designed from the ground up for AI-assisted development. Unlike traditional editors with AI bolted on, Tyck treats AI as a first-class citizen with deep integration throughout the editing experience.

### Key Features

- **AI-Native Architecture** — AI agents are integrated at the core, not as an afterthought
- **Tapp Extension System** — Build powerful extensions using WebAssembly (WASM)
- **Agent-First APIs** — Extensions can inject context, register tools, and spawn agent sessions
- **Multi-Storage Options** — Session, JSON, and SQLite storage for extensions
- **Sandboxed Security** — WASM-based isolation with capability-based permissions

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│ AwarenessBar - File tabs, Editor/Review toggle, AI status          │
├──────────────┬──────────────────────────┬───────────────────────────┤
│ ContextZone  │ FocusZone                │ InsightZone               │
│ (left panel) │ (center - Monaco editor) │ (right - Agent terminal)  │
├──────────────┴──────────────────────────┴───────────────────────────┤
│ TerminalPanel (optional, resizable)                                  │
├─────────────────────────────────────────────────────────────────────┤
│ CommandRail - Shell input, Context/Focus buttons                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
tyck/
├── src-tauri/              # Rust backend (Tauri)
│   ├── src/
│   │   ├── apps/           # Tapp extension system
│   │   │   ├── manager.rs  # Extension lifecycle management
│   │   │   ├── registry.rs # Extension registry
│   │   │   ├── storage.rs  # Session/JSON/SQLite storage
│   │   │   ├── tools.rs    # Agent tool registration
│   │   │   └── ...
│   │   ├── wasm/           # WASM runtime (Wasmtime)
│   │   └── commands/       # Tauri commands
│   └── Cargo.toml
├── packages/
│   ├── tapp/               # Tapp SDK (Rust crate)
│   ├── tapp-cli/           # CLI tool for building extensions
│   ├── tapp-macros/        # Procedural macros for Tapp
│   └── examples/           # Example Tapp extensions
│       ├── minimal-app/
│       └── database-explorer/
├── web/                    # Documentation website
└── docs/                   # Research & design documents
```

## Tapp Extension System

Tapp (Tyck App) is a constrained, secure, and performant extension system. Extensions are written in Rust and compiled to WebAssembly for sandboxed execution.

### Why Tapp?

| Traditional Plugins | Tapp Extensions |
|---------------------|-----------------|
| Every extension brings its own framework | Single consistent runtime |
| Inconsistent UX across extensions | Unified look & feel |
| Large bundle sizes | Shared components, tiny bundles |
| Security harder to audit | WASM sandbox with capabilities |
| Arbitrary choices | Opinionated, best practices built-in |

### Quick Start

1. Install the Tapp CLI:
```bash
cargo install --path packages/tapp-cli
```

2. Create a new extension:
```bash
tapp init my-extension
cd my-extension
```

3. Build and run:
```bash
tapp dev
```

### Example Extension

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
        ui::panel("Counter").children([
            ui::text(format!("Count: {}", self.count)),
            ui::hstack([
                ui::button("-").on_click("decrement"),
                ui::button("+").on_click("increment"),
            ]),
        ])
    }
}
```

### Permissions

Extensions declare required permissions in their manifest:

| Permission | Description | Risk Level |
|------------|-------------|------------|
| `fs:read` | Read files in project | Low |
| `fs:write` | Write files in project | Medium |
| `network:fetch` | HTTP requests to declared origins | Medium |
| `storage:persistent` | JSON + SQLite storage | Low |
| `agent:inject` | Inject context to active agent | Low |
| `agent:tools` | Register tools for agent | Medium |
| `agent:spawn` | Spawn new agent sessions | Medium |

### Storage Options

Extensions have three storage options:

```rust
// Session storage (in-memory, cleared on unload)
self.storage.session_set("key", "value");

// JSON storage (persistent, simple key-value)
self.storage.json_set("config", &my_config)?;

// SQLite database (persistent, full SQL)
self.storage.sql_query("SELECT * FROM items", &[])?;
```

## Development

### Prerequisites

- **Rust** 1.75+ with the `wasm32-wasip2` target
- **Node.js** 20+
- **Tauri CLI** 2.0+

### Building

```bash
# Install Rust WASM target
rustup target add wasm32-wasip2

# Build the editor
cargo tauri build

# Run in development mode
cargo tauri dev
```

### Running Tests

```bash
# Run all tests
cargo test

# Run Tapp SDK tests
cargo test -p tapp

# Run CLI tests
cargo test -p tapp-cli
```

## Documentation

- [Tapp Development Guide](docs/tapp/development.md) — Complete guide to building extensions
- [Research Documents](docs/research/) — Architecture decisions and design exploration

## Contributing

We welcome contributions! Please see our contributing guidelines (coming soon).

### Areas We Need Help With

- [ ] Additional UI components for the Tapp framework
- [ ] More example extensions
- [ ] Documentation improvements
- [ ] Testing infrastructure
- [ ] Cross-platform testing (Windows, Linux)

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with:
- [Tauri](https://tauri.app) — Desktop application framework
- [Wasmtime](https://wasmtime.dev) — WebAssembly runtime
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) — Code editor component
