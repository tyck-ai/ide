# Plan 08: Missing UI Components

## Problem

7 of 32 node types defined in the SDK have no Svelte renderer: `markdown`, `tree`, `data_grid`, `virtual_list`, `toast`, `drawer`, `skeleton`, `avatar`. They fall through to a passthrough div. Real apps need `markdown`, `tree`, `data_grid`, and `virtual_list`.

## Goal

Implement the 4 most-needed components. Defer the 4 less-critical ones.

## Prerequisites

None — this is independent of the WASM bridge work.

## Scope

**Implement now (high value):**
1. `markdown` — Renders markdown text as HTML. Critical for documentation apps.
2. `tree` — Hierarchical expandable tree view. Critical for file explorers, AST viewers.
3. `data_grid` — Tabular data with headers, sortable columns. Critical for database explorers.
4. `virtual_list` — Efficiently renders large lists with windowing. Critical for performance.

**Defer (lower priority):**
5. `toast` — Notification popup (can use `alert` as workaround)
6. `drawer` — Side panel overlay (can use `modal` as workaround)
7. `skeleton` — Loading placeholder (can use `spinner` as workaround)
8. `avatar` — User avatar circle (can use `icon` or `image` as workaround)

## Implementation Steps

### 8a. Markdown Component

1. Add `marked` or `snarkdown` npm dependency (lightweight markdown parser)
2. Create `src/lib/components/tapp/nodes/TappMarkdown.svelte`
3. Parse markdown to HTML, sanitize with DOMPurify or manual sanitization
4. Render via `{@html sanitizedHtml}` — MUST sanitize to prevent XSS
5. Add to TappRenderer.svelte
6. Style with theme CSS variables

### 8b. Tree Component

1. Create `src/lib/components/tapp/nodes/TappTree.svelte`
2. Recursive component: each node has `label`, `children`, `expanded`, `icon`
3. Click to expand/collapse, dispatch `on_toggle` event with node path
4. Indent levels with CSS
5. Add to TappRenderer.svelte

### 8c. DataGrid Component

1. Create `src/lib/components/tapp/nodes/TappDataGrid.svelte`
2. Props: `columns` (array of {key, label, width}), `rows` (array of objects)
3. Fixed header, scrollable body
4. Click column header to sort (dispatch `on_sort` event)
5. Optional row selection (dispatch `on_select` event)
6. Add to TappRenderer.svelte

### 8d. VirtualList Component

1. Create `src/lib/components/tapp/nodes/TappVirtualList.svelte`
2. Props: `total_items`, `item_height`, `visible_count`
3. Only render items in the visible viewport
4. Use scroll position to calculate which items to show
5. Dispatch `on_render_range` event so WASM can provide items for the visible range
6. Add to TappRenderer.svelte

## Files Modified

- `src/lib/components/tapp/TappRenderer.svelte` (add 4 branches)
- `src/lib/components/tapp/nodes/TappMarkdown.svelte` (new)
- `src/lib/components/tapp/nodes/TappTree.svelte` (new)
- `src/lib/components/tapp/nodes/TappDataGrid.svelte` (new)
- `src/lib/components/tapp/nodes/TappVirtualList.svelte` (new)
- `src/lib/components/tapp/index.ts` (export new components)
- `package.json` (add markdown parser dependency)

## Estimated Complexity

Medium. Each component is self-contained. Markdown needs XSS care. VirtualList needs scroll math.
