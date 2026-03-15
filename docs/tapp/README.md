# Tapp Extension System Documentation

Tapp is the extension system for Tyck IDE, enabling developers and AI agents to build powerful extensions using Rust compiled to WebAssembly.

## Documentation Index

### Getting Started
- [Quick Start Guide](./quick-start.md) - Build your first Tapp in 5 minutes
- [Installation](./installation.md) - Install the tapp CLI and dependencies

### Development
- [App Development Guide](./development.md) - Comprehensive guide to building Tapps
- [UI Components Reference](./ui-components.md) - All available UI components
- [Agent Integration](./agent-integration.md) - Tools, hooks, and agent APIs

### Publishing & Distribution
- [Building & Publishing](./publishing.md) - How to publish to crates.io, Homebrew, etc.
- [App Store Submission](./app-store.md) - Submit your app to the Tyck App Store

### Reference
- [Manifest Reference](./manifest-reference.md) - Complete manifest.json specification
- [API Reference](./api-reference.md) - Full tapp crate API documentation
- [Examples](./examples/) - Complete example applications

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Tyck IDE (Host)                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  UI Layer   │  │ App Manager │  │   Agent Bridge      │  │
│  │  (Svelte)   │  │   (Rust)    │  │     (Rust)          │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                     │             │
│         └────────────────┼─────────────────────┘             │
│                          │                                   │
│                   ┌──────┴──────┐                            │
│                   │ WASM Host   │                            │
│                   │ (wasmtime)  │                            │
│                   └──────┬──────┘                            │
└──────────────────────────┼──────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────┴─────┐ ┌────┴────┐ ┌─────┴─────┐
        │  App A    │ │  App B  │ │  App C    │
        │  (WASM)   │ │ (WASM)  │ │  (WASM)   │
        └───────────┘ └─────────┘ └───────────┘
```

## Key Concepts

### Performance First
All business logic runs in Rust/WASM. The frontend is a thin rendering layer only.

### Agent-Native Development
Apps are designed to be built by AI agents:
- Rust complexity is handled by agents
- Boilerplate is generated automatically
- Compiler errors guide iteration
- Structured examples for pattern matching

### Security by Default
- WASM sandboxing with explicit capability grants
- Permission system enforced at runtime
- No ambient filesystem or network access
