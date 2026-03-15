# Plan 05: Editor Awareness — Project & Editor State from WASM

## Problem

Apps can't see what file is open, what's selected, what project root is, or what git branch is active. This limits apps to self-contained tools rather than context-aware extensions.

## Goal

Apps can query editor state (active file, selection, project root, git status) through a host-imported API.

## Prerequisites

Plan 01 (WIT Host Imports).

## Scope

- Add `host-editor` interface to WIT
- Implement host-side handlers that read from Svelte stores via Tauri state
- Add guest-side `Editor` API to the SDK

## Design

```wit
interface host-editor {
    get-project-root: func() -> result<string, string>;
    get-active-file: func() -> result<string, string>;  // JSON FileInfo or empty
    get-open-files: func() -> result<string, string>;  // JSON array
    get-selection: func() -> result<string, string>;  // JSON Selection or empty
    get-git-branch: func() -> result<string, string>;
    get-git-status: func() -> result<string, string>;  // JSON
}
```

## Implementation Steps

1. Add `host-editor` to `tapp.wit`
2. Create a shared state bridge on the Tauri side:
   - Add a `ProjectState` struct in `src-tauri/src/apps/` that mirrors the frontend stores
   - Frontend sends state updates to backend via Tauri events or commands
   - `set_project_root`, `set_active_file`, `set_selection`, `set_git_branch`
3. Implement host handler:
   - Read from `ProjectState` (behind `Arc<RwLock<>>`)
   - Return current values as JSON
4. Add `packages/tapp/src/editor.rs` guest-side:
   - `pub fn project_root() -> Result<String>`
   - `pub fn active_file() -> Result<Option<FileInfo>>`
   - `pub fn open_files() -> Result<Vec<FileInfo>>`
   - `pub fn selection() -> Result<Option<Selection>>`
   - `pub fn git_branch() -> Result<String>`
5. Wire frontend stores to backend:
   - In `+page.svelte` or relevant stores, emit state changes to backend
   - Use `invoke('set_editor_state', { ... })` on store change
6. Test: app that displays "Currently editing: {filename}" and updates when user switches files

## Challenge

The biggest challenge is syncing frontend store state to the backend. Options:
- **Push model**: Frontend emits on every change (reactive subscription)
- **Pull model**: Backend queries frontend via Tauri events when WASM asks
- **Hybrid**: Cache last-known state in backend, frontend pushes updates

Push model is simplest and most responsive.

## Files Modified

- `packages/tapp/wit/tapp.wit`
- `src-tauri/src/wasm/instance.rs`
- `src-tauri/src/apps/` (new `project_state.rs`)
- `packages/tapp/src/editor.rs` (new)
- `packages/tapp/src/lib.rs`
- `src/routes/+page.svelte` (add state sync)
- `src/lib/stores/editor.ts` (add backend sync)

## Estimated Complexity

Medium-High. The state synchronization between frontend stores and backend is the hard part.
