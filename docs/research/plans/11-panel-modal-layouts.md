# Plan 11: Panel and Modal Layout Modes

## Problem

Only `full` and `sidebar` layouts work. `panel` (bottom panel like terminal) and `modal` (overlay dialog) are defined in the manifest spec but not implemented in `+page.svelte`.

## Goal

Apps can use `layout: "panel"` to render in the terminal panel area, or `layout: "modal"` to render as an overlay dialog.

## Prerequisites

None — independent work.

## Implementation Steps

### Panel Layout

1. In `+page.svelte`, when `$activeApp.layout === 'panel'`:
   - Don't replace any zone — keep ContextZone + FocusZone + InsightZone
   - Render `TappContainer` in the terminal panel area (where `TerminalPanel` goes)
   - If terminal is also visible, split the bottom area
2. Add CSS for panel layout container

### Modal Layout

1. In `+page.svelte`, when `$activeApp.layout === 'modal'`:
   - Don't replace any zone
   - Render `TappContainer` as a centered overlay with backdrop
   - Use existing `.tapp-container--modal` CSS (already defined in TappContainer)
   - Close on Escape or backdrop click
2. TappContainer already has `--modal` CSS variant

## Files Modified

- `src/routes/+page.svelte`
- `src/lib/components/tapp/TappContainer.svelte` (minor tweaks)

## Estimated Complexity

Low. Most of the CSS already exists.
