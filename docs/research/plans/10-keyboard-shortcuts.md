# Plan 10: App Keyboard Shortcuts

## Problem

The manifest supports `shortcuts` field but they're never registered. Apps can't define hotkeys like `Cmd+Shift+D` to open.

## Goal

Apps can declare keyboard shortcuts in their manifest that trigger app actions.

## Prerequisites

None — independent work.

## Scope

Small. Read shortcuts from manifest on install/start, register with the global keyboard handler.

## Implementation Steps

1. When an app is started, read `manifest.shortcuts`
2. Register each shortcut with the frontend keyboard handler (`+page.svelte` `onKeydown`)
3. When shortcut fires, either:
   - `"open"` → activate the app
   - Custom action name → dispatch as an action to the running app
4. When app is stopped, unregister shortcuts
5. Show shortcuts in app launcher next to app names

## Files Modified

- `src-tauri/src/apps/manifest.rs` (add `shortcuts` field to Manifest struct)
- `src-tauri/src/apps/commands.rs` (expose shortcuts in AppInfo)
- `src/lib/stores/tapp.ts` (track active shortcuts)
- `src/routes/+page.svelte` (register in keydown handler)
- `src/lib/components/AppLauncher.svelte` (display shortcuts)

## Estimated Complexity

Low.
