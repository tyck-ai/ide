# tapp-macros

Procedural macros for the Tapp extension SDK.

This crate provides the derive macros and attribute macros used by the `tapp` crate. You typically don't need to depend on this directly - it's re-exported through `tapp`.

## Macros

### `#[tapp::app]`

Marks a struct as a Tapp application and generates the necessary WASM exports.

```rust
use tapp::prelude::*;

#[tapp::app]
#[derive(Default)]
pub struct MyApp {
    // state fields
}

impl App for MyApp {
    // required methods
}
```

### `#[tapp::tools]`

Marks an impl block as containing agent tools.

```rust
#[tapp::tools]
impl MyApp {
    #[tool(name = "my_tool", description = "Does something useful")]
    fn my_tool(&self, args: serde_json::Value) -> ToolResult {
        ToolResult::ok()
    }
}
```

### `#[tapp::hooks]`

Marks an impl block as containing agent hooks.

```rust
#[tapp::hooks]
impl MyApp {
    #[hook(on_before_input)]
    fn enrich_input(&self, data: &serde_json::Value) -> HookResult {
        HookResult::pass_through()
    }
}
```

## Internal Details

This crate generates:
- WASM export functions for the app lifecycle
- Tool discovery and registration
- Hook registration and dispatch
- Serialization/deserialization glue

## License

MIT
