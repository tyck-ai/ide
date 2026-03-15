# Plan 03: Filesystem API — Project File Access from WASM

## Problem

Apps can't read project files. A database explorer can't read `.env`, a dependency analyzer can't read `package.json`. The WASI preopen only gives the app's own data directory.

## Goal

Apps with `fs:read` or `fs:write` permissions can read/write files in the project directory through a host-imported API.

## Prerequisites

Plan 01 (WIT Host Imports).

## Scope

- Add `host-fs` interface to WIT
- Implement host-side filesystem handlers with permission checks
- Add guest-side `Fs` API to the SDK
- Path validation: restrict to project root, reject traversal

## Design

```wit
interface host-fs {
    read-file: func(path: string) -> result<string, string>;
    read-file-bytes: func(path: string) -> result<list<u8>, string>;
    write-file: func(path: string, content: string) -> result<_, string>;
    exists: func(path: string) -> result<bool, string>;
    read-dir: func(path: string) -> result<string, string>;  // JSON array
    stat: func(path: string) -> result<string, string>;  // JSON object
    glob: func(pattern: string) -> result<string, string>;  // JSON array of paths
}
```

## Implementation Steps

1. Add `host-fs` to `tapp.wit` as an import
2. Implement host handler in `src-tauri/src/wasm/instance.rs`:
   - All operations scoped to the project root (from `projectRoot` store)
   - Check `fs:read` permission before reads
   - Check `fs:write` permission before writes
   - Canonicalize paths + verify they stay under project root
   - Reject symlinks that escape the project tree
3. Add `packages/tapp/src/fs.rs` guest-side module:
   - `pub fn read_file(path: &str) -> Result<String>`
   - `pub fn write_file(path: &str, content: &str) -> Result<()>`
   - `pub fn exists(path: &str) -> Result<bool>`
   - `pub fn read_dir(path: &str) -> Result<Vec<DirEntry>>`
   - `pub fn glob(pattern: &str) -> Result<Vec<String>>`
4. Export from `packages/tapp/src/lib.rs` and add to prelude
5. Test: app that reads `package.json` and displays dependencies

## Security

- All paths relative to project root
- Absolute paths rejected
- `..` traversal checked after join
- `fs:system` permission required for paths outside project
- Per-operation permission checks (read vs write)

## Files Modified

- `packages/tapp/wit/tapp.wit`
- `src-tauri/src/wasm/instance.rs`
- `packages/tapp/src/fs.rs` (new)
- `packages/tapp/src/lib.rs`

## Estimated Complexity

Medium. Main challenge is getting the project root path into the WASM host context.
