# Tapp System — Implementation Plans Overview

## Execution Order

These plans are ordered by dependency and impact. Each can be tackled by a separate agent.

### Foundation (must be first)

| # | Plan | Complexity | Depends On |
|---|------|-----------|------------|
| 01 | [WIT Host Imports](01-wit-host-imports.md) | Medium-High | — |

### Core APIs (unlock real apps, do in parallel after 01)

| # | Plan | Complexity | Depends On |
|---|------|-----------|------------|
| 02 | [Storage Bridge](02-storage-bridge.md) | Medium | 01 |
| 03 | [Filesystem API](03-filesystem-api.md) | Medium | 01 |
| 04 | [Network Bridge](04-network-bridge.md) | Low-Medium | 01 |
| 05 | [Editor Awareness](05-editor-awareness.md) | Medium-High | 01 |

### Agent Integration (the differentiator)

| # | Plan | Complexity | Depends On |
|---|------|-----------|------------|
| 06 | [Agent Context Injection](06-agent-inject.md) | Medium | 01 |
| 07 | [Agent Session Spawning](07-agent-spawn.md) | High | 01, 06 |

### UI & DX (independent, can be done anytime)

| # | Plan | Complexity | Depends On |
|---|------|-----------|------------|
| 08 | [Missing UI Components](08-missing-ui-components.md) | Medium | — |
| 09 | [TypeScript SDK](09-typescript-sdk.md) | High | 01 (stable WIT) |
| 10 | [Keyboard Shortcuts](10-keyboard-shortcuts.md) | Low | — |
| 11 | [Panel & Modal Layouts](11-panel-modal-layouts.md) | Low | — |

## Recommended Execution Waves

**Wave 1** (unblocks everything):
- Plan 01: WIT Host Imports

**Wave 2** (parallel, after 01):
- Plan 02: Storage Bridge
- Plan 03: Filesystem API
- Plan 04: Network Bridge
- Plan 08: Missing UI Components (independent)
- Plan 10: Keyboard Shortcuts (independent)
- Plan 11: Panel & Modal Layouts (independent)

**Wave 3** (parallel, after 01):
- Plan 05: Editor Awareness
- Plan 06: Agent Inject

**Wave 4** (after 06):
- Plan 07: Agent Spawn

**Wave 5** (after WIT is stable):
- Plan 09: TypeScript SDK

## What this unlocks

After all plans: developers can build a full **Database Explorer** app that:
- Reads `.env` for connection strings (Plan 03)
- Stores connection history (Plan 02)
- Makes HTTP requests to database APIs (Plan 04)
- Shows query results in a data grid (Plan 08)
- Registers `query_database` tool for the AI agent (already works)
- Injects SQL context into the agent (Plan 06)
- Spawns a background agent to analyze schema (Plan 07)
- Opens with `Cmd+Shift+D` (Plan 10)
- Shows as a sidebar panel alongside the editor (already works)
- Written in TypeScript (Plan 09)
