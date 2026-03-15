# Plan 01: WIT Host Imports — The Bridge Layer

## Problem

The WIT interface only defines **exports** (app→host: init, render, handle, shutdown, tools, hooks). There are no **imports** (host→app), so WASM apps cannot call back into the host for storage, filesystem, network, or editor APIs. This is the root cause of shortcomings 1, 2, and 4.

## Goal

Add host-imported functions to the WIT interface so WASM guest code can call host APIs. This is the foundation that all other API plans depend on.

## Scope

- Modify `packages/tapp/wit/tapp.wit` to add imported interfaces
- Implement the host-side import handlers in `src-tauri/src/wasm/instance.rs`
- Wire the imports into the wasmtime linker
- Update the `bindgen!` macro usage to include imports

## Design

Add these imported interfaces to the WIT world:

```wit
interface host-storage {
    session-get: func(key: string) -> result<string, string>;
    session-set: func(key: string, value: string) -> result<_, string>;
    session-delete: func(key: string) -> result<_, string>;
    json-get: func(key: string) -> result<string, string>;
    json-set: func(key: string, value: string) -> result<_, string>;
    json-delete: func(key: string) -> result<_, string>;
    sql-execute: func(sql: string, params: string) -> result<string, string>;
    sql-query: func(sql: string, params: string) -> result<string, string>;
}

interface host-log {
    log: func(level: string, message: string);
}

world tapp-app {
    import host-storage;
    import host-log;
    export app;
    export tools;
    export hooks;
}
```

## Implementation Steps

1. Update `packages/tapp/wit/tapp.wit` with the new imported interfaces
2. Update `src-tauri/src/wasm/instance.rs`:
   - Change `bindgen!` to pick up imports
   - Implement the `host-storage` trait on a struct that holds a reference to `AppStorage`
   - Implement the `host-log` trait
3. Update `WasmInstance::new()` to register import implementations with the linker
4. Update `packages/tapp/src/storage.rs` guest-side to call the imported functions instead of returning errors
5. Test: a minimal app that does `session_set("key", "value")` then `session_get("key")` and displays the result

## Files Modified

- `packages/tapp/wit/tapp.wit`
- `src-tauri/src/wasm/instance.rs`
- `packages/tapp/src/storage.rs`
- `packages/tapp-macros/src/lib.rs` (may need to update generated code)

## Dependencies

None — this is the foundation plan.

## Estimated Complexity

Medium-High. The wasmtime component model import binding is the tricky part.
