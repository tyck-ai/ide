# Manifest Reference

Complete specification for the `manifest.json` file that every Tapp requires.

## Basic Structure

```json
{
  "id": "my-app",
  "name": "My Application",
  "version": "1.0.0",
  "description": "A helpful description of what this app does",
  "author": "Your Name <you@example.com>",
  "permissions": ["storage:persistent"],
  "ui": {
    "layout": "full"
  }
}
```

## Required Fields

### id
**Type**: `string`
**Pattern**: `^[a-z0-9-]+$`

Unique identifier for the app. Used for installation paths, storage, and registry.

```json
{ "id": "my-database-explorer" }
```

### name
**Type**: `string`
**Max Length**: 50

Human-readable display name.

```json
{ "name": "Database Explorer" }
```

### version
**Type**: `string`
**Pattern**: `^\d+\.\d+\.\d+$`

Semantic version (semver).

```json
{ "version": "1.2.3" }
```

### permissions
**Type**: `array<string>`

List of permissions the app requires. See [Permissions](#permissions-reference) section.

```json
{ "permissions": ["storage:persistent", "agent:tools"] }
```

## Optional Fields

### description
**Type**: `string`

Description shown in app listings.

```json
{ "description": "Browse and query your databases directly from Tyck" }
```

### author
**Type**: `string`

Author name and optional email.

```json
{ "author": "Jane Developer <jane@example.com>" }
```

### homepage
**Type**: `string`

URL to app homepage or documentation.

```json
{ "homepage": "https://example.com/my-app" }
```

### repository
**Type**: `string`

URL to source code repository.

```json
{ "repository": "https://github.com/username/my-app" }
```

### license
**Type**: `string`

SPDX license identifier.

```json
{ "license": "MIT" }
```

### keywords
**Type**: `array<string>`

Keywords for discovery/search.

```json
{ "keywords": ["database", "sql", "explorer"] }
```

### icon
**Type**: `string`

Path to icon file (relative to manifest).

```json
{ "icon": "assets/icon.svg" }
```

### ui
**Type**: `object`

UI configuration. See [UI Configuration](#ui-configuration) section.

```json
{
  "ui": {
    "layout": "sidebar",
    "minWidth": 300
  }
}
```

### network
**Type**: `object`

Network access configuration. Required for `network:fetch` permission.

```json
{
  "network": {
    "allowedHosts": [
      "api.example.com:443",
      "*.database.example.com:5432"
    ]
  }
}
```

## Permissions Reference

### Storage Permissions

| Permission | Description |
|------------|-------------|
| `storage:session` | In-memory session storage only |
| `storage:persistent` | JSON file + SQLite database access |

### Filesystem Permissions

| Permission | Description |
|------------|-------------|
| `fs:read` | Read files in workspace |
| `fs:write` | Write files in workspace |
| `fs:system` | Access files outside workspace (restricted) |

### Network Permissions

| Permission | Description |
|------------|-------------|
| `network:fetch` | HTTP/HTTPS to allowed hosts only |
| `network:unrestricted` | Full network access (TCP, TLS, WebSocket) |

### Agent Permissions

| Permission | Description |
|------------|-------------|
| `agent:inject` | Send text to active agent session |
| `agent:tools` | Expose tools for agents to call |
| `agent:hooks` | Intercept agent input/output |
| `agent:spawn` | Create new agent sessions |

## UI Configuration

### layout
**Type**: `string`
**Values**: `"full"` | `"sidebar"` | `"panel"` | `"modal"`
**Default**: `"full"`

```json
{
  "ui": {
    "layout": "sidebar"
  }
}
```

| Layout | Description |
|--------|-------------|
| `full` | Replace main content area (ContextZone + FocusZone) |
| `sidebar` | Replace context panel only |
| `panel` | Add as panel below terminal |
| `modal` | Display as modal overlay |

### minWidth
**Type**: `number`

Minimum width in pixels (for sidebar/panel).

```json
{
  "ui": {
    "layout": "sidebar",
    "minWidth": 250
  }
}
```

### minHeight
**Type**: `number`

Minimum height in pixels (for panel).

```json
{
  "ui": {
    "layout": "panel",
    "minHeight": 200
  }
}
```

### defaultWidth
**Type**: `number`

Default width in pixels.

### defaultHeight
**Type**: `number`

Default height in pixels.

## Network Configuration

When using `network:fetch`, you must specify allowed hosts:

```json
{
  "permissions": ["network:fetch"],
  "network": {
    "allowedHosts": [
      "api.github.com:443",
      "api.gitlab.com:443",
      "*.amazonaws.com:443"
    ]
  }
}
```

### Host Pattern Syntax

- `host:port` - Exact match
- `*.host:port` - Subdomain wildcard
- `host:*` - Any port (not recommended)

## Complete Example

```json
{
  "$schema": "https://tyck.dev/schemas/tapp-manifest.json",
  "id": "database-explorer",
  "name": "Database Explorer",
  "version": "2.1.0",
  "description": "Browse, query, and manage databases directly from Tyck IDE",
  "author": "Tyck Team <team@tyck.dev>",
  "homepage": "https://tyck.dev/apps/database-explorer",
  "repository": "https://github.com/tyck-dev/database-explorer",
  "license": "MIT",
  "keywords": ["database", "sql", "postgresql", "mysql", "sqlite"],
  "icon": "assets/icon.svg",
  "permissions": [
    "storage:persistent",
    "network:unrestricted",
    "agent:tools",
    "agent:hooks"
  ],
  "ui": {
    "layout": "sidebar",
    "minWidth": 300,
    "defaultWidth": 400
  }
}
```

## JSON Schema

For IDE autocompletion, reference the schema:

```json
{
  "$schema": "https://tyck.dev/schemas/tapp-manifest.json"
}
```

## Validation

The `tapp` CLI validates manifests on build:

```bash
tapp build
# Error: manifest.json: 'id' must match pattern ^[a-z0-9-]+$
# Error: manifest.json: unknown permission 'invalid:perm'
```

## Migration Guide

### From v0.1 to v0.2

1. **Renamed permissions**:
   - `storage:json` → `storage:persistent`
   - `storage:sql` → (included in `storage:persistent`)

2. **New UI configuration**:
   - `layout` moved under `ui` object
   - Added `minWidth`, `minHeight` options

### Breaking Changes

- `id` now requires lowercase with hyphens only
- `permissions` is now required (empty array if none needed)
