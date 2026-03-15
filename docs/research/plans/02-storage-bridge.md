# Plan 02: Storage Bridge — Connect Guest SDK to Host Storage

## Problem

`AppStorage` on the host has full JSON and SQLite support, but the guest SDK (`packages/tapp/src/storage.rs`) returns errors for all persistent operations because there's no WASM→host bridge.

## Goal

Apps can call `json_get`/`json_set`/`sql_execute`/`sql_query` from Rust WASM and have them execute on the host's `AppStorage`.

## Prerequisites

Plan 01 (WIT Host Imports) must be completed first.

## Scope

- Implement the `host-storage` import handler on the host side
- Update the guest SDK `storage.rs` to call WIT imports instead of returning errors
- Ensure permission checks still work (apps without `storage:persistent` get denied)

## Implementation Steps

1. In `src-tauri/src/wasm/instance.rs`, implement the `HostStorage` trait:
   - Each method looks up the app's `AppStorage` from the running apps map
   - JSON methods delegate to `AppStorage::json_get/set/delete`
   - SQL methods delegate to `AppStorage::sql_execute/query`
   - Session methods delegate to `AppStorage::session_get/set/delete`
2. In `packages/tapp/src/storage.rs`, replace the WASM-target stubs:
   - `json_get` → call `host_storage::json_get` (the WIT import)
   - `json_set` → call `host_storage::json_set`
   - Same for sql_execute, sql_query, session_*
3. Test with an app that:
   - Stores a counter in JSON persistent storage
   - Reads it back on next render
   - Survives app stop/start

## Challenge

The host import handler needs access to `AppStorage`, but the wasmtime store's data (`WasmHostState`) doesn't currently hold a reference to it. Options:
- Store an `Arc<Mutex<AppStorage>>` in `WasmHostState`
- Pass app_id through the store and look up storage in a global map

## Files Modified

- `src-tauri/src/wasm/instance.rs`
- `src-tauri/src/wasm/host.rs` (WasmHostState may need storage ref)
- `packages/tapp/src/storage.rs`

## Estimated Complexity

Medium. The host-side storage already works — this is plumbing.
