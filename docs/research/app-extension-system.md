# Tyck App Extension System - Research Document

## Executive Summary

This document outlines the research and design for **Tapp** (Tyck App) - a constrained, secure, and performant app extension system for Tyck IDE. Apps run in isolated webviews, use the Tapp Framework for UI/backend, integrate deeply with AI agents (including spawning sessions), and have access to session and persistent storage.

**Key Design Decisions:**
- **Constrained Framework**: Tapp Framework (not arbitrary React/Vue/etc.) for consistent UX and performance
- **Webview Isolation**: Security via sandboxed webviews with message-passing API
- **Agent-First**: Apps can inject context, register tools, spawn sessions, and hook into agent I/O
- **Native Tools**: Tapp Tools interface (simpler than MCP) with optional MCP export

---

## 1. Current Architecture Analysis

### 1.1 UI Layout Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│ AwarenessBar (36px) - File tabs, Editor/Review toggle, AI status    │
├──────────────┬──────────────────────────┬───────────────────────────┤
│ ContextZone  │ FocusZone                │ InsightZone               │
│ (left panel) │ (center - Monaco editor) │ (right - Agent terminal)  │
│ 160-500px    │ flex: 1                  │ 200-600px                 │
├──────────────┴──────────────────────────┴───────────────────────────┤
│ TerminalPanel (optional, resizable)                                  │
├─────────────────────────────────────────────────────────────────────┤
│ CommandRail - Shell input, Context/Focus buttons                     │
├─────────────────────────────────────────────────────────────────────┤
│ GitStatusBar (when in repo)                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 Current State Management

| Store | Purpose | Persistence |
|-------|---------|-------------|
| `editor` | Open files, cursor, selection | Session only |
| `layout` | UI visibility states | Session only |
| `settings` | App settings, theme | `~/.tyck/settings.json` |
| `agentTerminal` | Agent sessions | Session only |
| `sessionReview` | Worktree diffs, decisions | `~/.tyck/reviews/` |
| `git` | Git state | Session only |

### 1.3 Communication Patterns

1. **Frontend → Backend**: `invoke()` for Tauri commands
2. **Backend → Frontend**: `emit()` for Tauri events
3. **Component ↔ Component**: Svelte stores (reactive)

### 1.4 Current Extension Points

- **Agent Providers**: Hardcoded list in `known_providers()`
- **Themes**: Built-in + custom JSON files in `~/.tyck/themes/`
- **No plugin/app API exists**

---

## 2. Tapp Framework Design

### 2.1 Why a Constrained Framework?

| Full Freedom (React/Vue/etc.) | Tapp Framework |
|-------------------------------|----------------|
| Every app brings its own framework | Single consistent runtime |
| Inconsistent UX across apps | Unified look & feel |
| Large bundle sizes (each app ships React) | Shared components, tiny app bundles |
| Performance varies wildly | Guaranteed performance baseline |
| Security harder to audit | Smaller API surface to secure |
| App developers make arbitrary choices | Opinionated, best practices built-in |

### 2.2 Tapp UI Components

The Tapp Framework provides a declarative, reactive UI system:

```typescript
import { App, View, Panel, Text, Button, List, useStore } from 'tapp';
import './styles/app.css';

const app = App.create({
  name: 'database-explorer',
  
  setup(tyck) {
    const items = useStore([]);
    const loading = useStore(false);
    
    return View({ class: 'app-root' }, [
      Panel({ title: 'Database Explorer' }, [
        List({
          items: items.value,
          renderItem: (item) => Text({ content: item.name })
        }),
        Button({
          label: 'Refresh',
          loading: loading.value,
          onClick: async () => {
            loading.value = true;
            items.value = await fetchItems();
            loading.value = false;
          }
        })
      ])
    ]);
  }
});

export default app;
```

#### Component Library

| Category | Components |
|----------|------------|
| **Layout** | `View`, `Panel`, `Tabs`, `Split`, `Scroll`, `Modal`, `Drawer` |
| **Content** | `Text`, `Code`, `Markdown`, `Icon`, `Image`, `Badge`, `Avatar` |
| **Input** | `Button`, `Input`, `TextArea`, `Select`, `Checkbox`, `Toggle`, `Slider`, `DatePicker` |
| **Data** | `List`, `VirtualList`, `Tree`, `Table`, `DataGrid` |
| **Feedback** | `Toast`, `Progress`, `Spinner`, `Empty`, `Skeleton`, `Alert` |
| **Navigation** | `Menu`, `Breadcrumb`, `Pagination` |

#### Reactivity System

```typescript
import { useStore, useComputed, useEffect } from 'tapp';

// Reactive state
const count = useStore(0);
const doubled = useComputed(() => count.value * 2);

// Side effects
useEffect(() => {
  console.log('Count changed:', count.value);
});

// Async data
const { data, loading, error, refetch } = useAsync(() => 
  tyck.fs.readFile('package.json')
);
```

### 2.3 Tapp Backend (WASM)

For apps needing Rust backend logic:

```rust
use tapp_backend::prelude::*;

#[tapp::app]
pub struct DatabaseExplorer {
    connections: Vec<Connection>,
}

#[tapp::app]
impl App for DatabaseExplorer {
    fn new() -> Self {
        Self { connections: vec![] }
    }
    
    fn init(&mut self, ctx: &Context) -> Result<()> {
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        for conn in &mut self.connections {
            conn.close()?;
        }
        Ok(())
    }
    
    fn handle(&mut self, action: Action) -> Result<Response> {
        match action.name.as_str() {
            "connect" => {
                let conn_string: String = action.get("connectionString")?;
                let conn = Connection::open(&conn_string)?;
                self.connections.push(conn);
                Ok(Response::ok())
            }
            "query" => {
                let sql: String = action.get("sql")?;
                let rows = self.connections[0].query(&sql)?;
                Ok(Response::json(json!({ "rows": rows })))
            }
            _ => Ok(Response::not_found())
        }
    }
}

// Tools exposed to agent
#[tapp::tools]
impl DatabaseExplorer {
    /// Execute a SQL query against the connected database
    #[tool(name = "query_database")]
    fn query_database(&mut self, query: String, database: Option<String>) -> Result<ToolResult> {
        let rows = self.connections[0].query(&query)?;
        Ok(ToolResult::json(json!({
            "rows": rows,
            "count": rows.len()
        })))
    }
}
```

### 2.4 Theme Integration

Apps inherit Tyck's CSS variables automatically:

```css
/* Apps use Tyck's theme variables */
.my-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.my-button {
  background: var(--color-accent);
  color: var(--color-on-accent);
}

/* Apps can define their own variables that layer on top */
.app-root {
  --app-sidebar-width: 200px;
}
```

---

## 3. App Structure & Manifest

### 3.1 Directory Structure

```
my-app/
├── manifest.json           # App metadata, permissions, entry points
├── src/
│   ├── app.ts              # Main app entry
│   ├── components/         # UI components
│   │   └── Main.ts
│   └── styles/
│       └── app.css
├── backend/                # Optional Rust WASM backend
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── assets/
│   └── icon.svg
├── tapp.config.ts          # Build configuration
├── package.json
├── tsconfig.json
└── README.md
```

### 3.2 Manifest Schema

```json
{
  "id": "database-explorer",
  "name": "Database Explorer",
  "version": "1.0.0",
  "description": "Connect and explore databases from Tyck",
  "author": "Your Name <you@example.com>",
  "icon": "assets/icon.svg",
  "license": "MIT",
  "repository": "https://github.com/you/database-explorer",
  
  "permissions": [
    "fs:read",
    "fs:write",
    "network:fetch",
    "storage:persistent",
    "agent:inject",
    "agent:tools",
    "agent:spawn"
  ],
  
  "ui": {
    "entry": "dist/app.js",
    "style": "dist/app.css",
    "layout": "full"
  },
  
  "backend": {
    "wasm": "dist/backend.wasm"
  },
  
  "shortcuts": [
    { "key": "CmdOrCtrl+Shift+D", "action": "open" },
    { "key": "CmdOrCtrl+Shift+Q", "action": "query" }
  ],
  
  "minTyckVersion": "0.2.0"
}
```

### 3.3 Permissions Reference

| Permission | Description | Risk Level |
|------------|-------------|------------|
| `fs:read` | Read files in project | Low |
| `fs:write` | Write files in project | Medium |
| `fs:system` | Access files outside project | High |
| `network:fetch` | HTTP requests to declared origins | Medium |
| `network:unrestricted` | Any network access | High |
| `storage:session` | In-memory session storage | Low |
| `storage:persistent` | JSON + SQLite storage | Low |
| `agent:inject` | Inject context to active agent | Low |
| `agent:tools` | Register tools for agent | Medium |
| `agent:hooks` | Intercept agent I/O | High |
| `agent:spawn` | Spawn new agent sessions | Medium |
| `apps:communicate` | Inter-app messaging (v2) | Medium |

---

## 4. Agent Integration

### 4.1 Inject Context to Active Session

Apps can inject text/context into the user's active agent session:

```typescript
// Simple text injection
await tyck.agent.inject('Check this SQL query for issues:\n' + sqlQuery);

// Inject file with context
await tyck.agent.injectFile('/src/utils.ts', {
  selection: { start: 10, end: 25 },
  prompt: 'Refactor this function'
});

// Inject code block
await tyck.agent.injectCodeBlock(generatedCode, 'typescript');
```

### 4.2 Spawn Agent Sessions

Apps can spawn and manage their own agent sessions:

```typescript
interface AgentAPI {
  // Get active session (user's main session)
  getActiveSession(): Promise<AgentSession>;
  
  // Inject into active session
  inject(text: string): Promise<void>;
  injectFile(path: string, options?: InjectOptions): Promise<void>;
  injectCodeBlock(code: string, language: string): Promise<void>;
  
  // Spawn new session (app-managed)
  spawn(options: SpawnOptions): Promise<AgentSession>;
  
  // List all sessions (user's + app's)
  getSessions(): Promise<AgentSession[]>;
}

interface SpawnOptions {
  provider?: string;           // 'claude-code' | 'codex' | etc.
  systemPrompt?: string;       // Custom system prompt
  cwd?: string;                // Working directory
  visible?: boolean;           // Show in InsightZone? (default: false)
  env?: Record<string, string>;
  name?: string;               // Display name for session
}

interface AgentSession {
  id: string;
  name: string;
  provider: string;
  status: 'idle' | 'running' | 'exited';
  visible: boolean;
  
  // Send input
  send(text: string): Promise<void>;
  
  // Stream output
  onOutput(callback: (chunk: string) => void): Unsubscribe;
  
  // Wait for agent to finish current task
  waitForIdle(): Promise<void>;
  
  // Get full conversation transcript
  getTranscript(): Promise<Message[]>;
  
  // Control visibility
  show(): Promise<void>;
  hide(): Promise<void>;
  focus(): Promise<void>;
  
  // Terminate session
  kill(): Promise<void>;
}
```

#### Session Visibility Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **Hidden** | App fully manages session, no UI in Tyck | Background tasks, automation |
| **Background Tab** | Session in InsightZone tabs, not active | Parallel tasks user can check |
| **Foreground** | Session becomes active in InsightZone | User-facing agent work |

#### Example: Background Code Analysis

```typescript
async function analyzeCodebase() {
  const session = await tyck.agent.spawn({
    provider: 'claude-code',
    systemPrompt: `You are a security auditor. Analyze code for:
      - SQL injection vulnerabilities
      - XSS vulnerabilities  
      - Hardcoded secrets
      - Insecure dependencies
      Output findings as JSON.`,
    visible: false,
    name: 'Security Scan'
  });

  const findings = [];
  
  session.onOutput((chunk) => {
    // Parse streaming output, update UI
    const parsed = tryParseJson(chunk);
    if (parsed) {
      findings.push(...parsed.findings);
      updateFindingsUI(findings);
    }
  });

  // Send files to analyze
  const files = await tyck.fs.glob('src/**/*.ts');
  for (const file of files) {
    const content = await tyck.fs.readFile(file);
    await session.send(`Analyze this file: ${file}\n\`\`\`typescript\n${content}\n\`\`\``);
    await session.waitForIdle();
  }

  // Cleanup
  await session.kill();
  
  return findings;
}
```

### 4.3 Tapp Tools (Agent Tool Registration)

Apps register tools using the native Tapp Tools interface:

```typescript
// Register a tool the agent can call
tyck.tools.register({
  name: 'query_database',
  description: 'Execute a SQL query and return results',
  parameters: {
    type: 'object',
    properties: {
      query: { type: 'string', description: 'SQL query to execute' },
      database: { type: 'string', description: 'Database name (optional)' },
      limit: { type: 'number', description: 'Max rows to return', default: 100 }
    },
    required: ['query']
  },
  handler: async (args) => {
    const { query, database, limit } = args;
    try {
      const result = await db.query(query, { database, limit });
      return {
        success: true,
        rows: result.rows,
        rowCount: result.rowCount,
        columns: result.columns
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }
});

// Unregister when done
tyck.tools.unregister('query_database');

// List registered tools
const tools = tyck.tools.list();
```

#### Tool Execution Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ Agent: "Let me query the database to check the users table..."  │
└─────────────────────┬───────────────────────────────────────────┘
                      │ tool_call: query_database
                      │ args: { query: "SELECT * FROM users LIMIT 5" }
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ Tapp Runtime                                                     │
│ - Validates tool exists and app has permission                   │
│ - Routes call to registered handler                              │
└─────────────────────┬───────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ Database Explorer App                                            │
│ - handler executes query                                         │
│ - Returns { rows: [...], rowCount: 5 }                          │
└─────────────────────┬───────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ Agent: "Found 5 users. The admin user was created on..."        │
└─────────────────────────────────────────────────────────────────┘
```

#### Why Tapp Tools over MCP?

| MCP | Tapp Tools |
|-----|------------|
| Separate process per app | In-process, zero overhead |
| JSON-RPC protocol overhead | Direct function calls |
| Complex server lifecycle | Simple register/unregister |
| App must handle MCP boilerplate | Just define handler |
| Works outside Tyck | Tyck-specific but simpler |

**Optional MCP Export**: Apps can opt to export tools as MCP for external use:

```typescript
// Export tools as MCP server (optional)
tyck.tools.exportAsMcp({ port: 3000 });
```

### 4.4 Agent Hooks

Apps can observe and optionally modify agent behavior:

```typescript
interface AgentHooksAPI {
  // Before input is sent to agent
  onBeforeInput(callback: (input: string, session: AgentSession) => string | void): Unsubscribe;
  
  // After agent produces output
  onAfterOutput(callback: (output: string, session: AgentSession) => void): Unsubscribe;
  
  // When agent calls any tool
  onToolCall(callback: (tool: string, args: object, session: AgentSession) => void): Unsubscribe;
  
  // Session lifecycle
  onSessionStart(callback: (session: AgentSession) => void): Unsubscribe;
  onSessionEnd(callback: (session: AgentSession) => void): Unsubscribe;
}

// Example: Log all agent interactions
tyck.hooks.onBeforeInput((input, session) => {
  console.log(`[${session.name}] Input:`, input);
});

tyck.hooks.onAfterOutput((output, session) => {
  console.log(`[${session.name}] Output:`, output);
});

// Example: Inject context automatically
tyck.hooks.onBeforeInput((input, session) => {
  if (input.includes('database')) {
    return input + '\n\nNote: Connected databases: ' + getConnectedDbs().join(', ');
  }
});
```

---

## 5. Storage System

### 5.1 Session Store (In-Memory)

Fast, ephemeral storage cleared when app unloads:

```typescript
interface SessionStore {
  get<T>(key: string): T | undefined;
  set<T>(key: string, value: T): void;
  delete(key: string): void;
  clear(): void;
  keys(): string[];
  
  // Reactive subscription
  subscribe<T>(key: string, callback: (value: T | undefined) => void): Unsubscribe;
}

// Usage
const session = tyck.storage.session();
session.set('currentQuery', 'SELECT * FROM users');
session.subscribe('currentQuery', (query) => {
  console.log('Query changed:', query);
});
```

### 5.2 JSON Store (Persistent Key-Value)

Simple file-based persistence for small data:

```typescript
interface JsonStore {
  get<T>(key: string): Promise<T | undefined>;
  set<T>(key: string, value: T): Promise<void>;
  delete(key: string): Promise<void>;
  has(key: string): Promise<boolean>;
  keys(): Promise<string[]>;
  values(): Promise<any[]>;
  entries(): Promise<[string, any][]>;
  clear(): Promise<void>;
}

// Usage
const json = await tyck.storage.json();
await json.set('connections', [
  { name: 'prod', host: 'db.example.com' },
  { name: 'dev', host: 'localhost' }
]);
const connections = await json.get('connections');
```

Storage location: `~/.tyck/apps/{app-id}/data/store.json`

### 5.3 SQLite Store (Persistent Database)

Full relational database for complex data:

```typescript
interface SqliteStore {
  execute(sql: string, params?: any[]): Promise<{ rowsAffected: number }>;
  query<T = any>(sql: string, params?: any[]): Promise<T[]>;
  queryOne<T = any>(sql: string, params?: any[]): Promise<T | null>;
  transaction<T>(fn: (tx: Transaction) => Promise<T>): Promise<T>;
  
  // Schema helpers
  tableExists(name: string): Promise<boolean>;
  getTableInfo(name: string): Promise<ColumnInfo[]>;
}

// Usage
const db = await tyck.storage.sqlite();

// Create tables
await db.execute(`
  CREATE TABLE IF NOT EXISTS queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sql TEXT NOT NULL,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_ms INTEGER,
    row_count INTEGER
  )
`);

// Insert
await db.execute(
  'INSERT INTO queries (sql, duration_ms, row_count) VALUES (?, ?, ?)',
  [query, duration, rowCount]
);

// Query
const recentQueries = await db.query<Query>(
  'SELECT * FROM queries ORDER BY executed_at DESC LIMIT 10'
);

// Transaction
await db.transaction(async (tx) => {
  await tx.execute('UPDATE queries SET archived = 1 WHERE id = ?', [id]);
  await tx.execute('INSERT INTO archive (query_id) VALUES (?)', [id]);
});
```

Storage location: `~/.tyck/apps/{app-id}/data/app.db`

---

## 6. App Installation & CLI

### 6.1 Package Format

Apps are distributed as `.tapp` files (zip archives):

```
my-app.tapp (zip)
├── manifest.json
├── dist/
│   ├── app.js           # Compiled Tapp UI bundle
│   ├── app.css          # Compiled styles
│   └── backend.wasm     # Compiled Tapp backend (optional)
└── assets/
    └── icon.svg
```

### 6.2 Installation Sources

| Source | Method |
|--------|--------|
| **Local Development** | `tyck apps dev ./my-app` |
| **Local File** | `tyck apps install ./my-app.tapp` |
| **URL** | `tyck apps install https://example.com/my-app.tapp` |
| **App Store (v2)** | UI browser + one-click install |

### 6.3 Installation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User initiates install                                        │
│    tyck apps install ./my-app.tapp                              │
└─────────────────────┬───────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Validate package                                              │
│    - Extract and parse manifest.json                             │
│    - Verify required fields                                      │
│    - Check minTyckVersion compatibility                          │
│    - Verify bundle integrity (signature check in v2)            │
└─────────────────────┬───────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Permission review dialog                                      │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │ "Database Explorer" requests:                            │  │
│    │                                                          │  │
│    │ ✓ Read files in your project                (fs:read)   │  │
│    │ ✓ Write files in your project               (fs:write)  │  │
│    │ ✓ Make network requests                 (network:fetch) │  │
│    │ ✓ Store persistent data           (storage:persistent)  │  │
│    │ ✓ Register agent tools                  (agent:tools)   │  │
│    │ ✓ Spawn agent sessions                  (agent:spawn)   │  │
│    │                                                          │  │
│    │                    [Cancel]  [Install]                   │  │
│    └─────────────────────────────────────────────────────────┘  │
└─────────────────────┬───────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Install to ~/.tyck/apps/{app-id}/                            │
│    - Extract files                                               │
│    - Create data/ directory for storage                         │
│    - Register in registry.json                                   │
│    - Set enabled: true                                           │
└─────────────────────┬───────────────────────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. App available                                                 │
│    - Appears in app launcher (Cmd+Shift+A)                      │
│    - Keyboard shortcuts active                                   │
│    - Tools registered with agent                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.4 CLI Reference

```bash
tyck apps

COMMANDS:
  init [name]              Create new app from template (interactive)
  dev <path>               Run app in development mode with hot reload
  build <path>             Build .tapp package
  install <path|url>       Install an app
  uninstall <id>           Uninstall an app
  list                     List installed apps
  enable <id>              Enable a disabled app
  disable <id>             Disable an app (keep installed)
  info <id>                Show app details and permissions
  logs <id>                View app logs
  open <id>                Open app in Tyck

OPTIONS:
  -h, --help               Show help
  -v, --version            Show version

EXAMPLES:
  tyck apps init                              # Interactive scaffolding
  tyck apps init my-app --template full       # Non-interactive
  tyck apps dev ./my-app                      # Development mode
  tyck apps build ./my-app -o my-app.tapp     # Build package
  tyck apps install ./my-app.tapp             # Install local
  tyck apps install https://example.com/x.tapp # Install from URL
  tyck apps list                              # Show installed apps
  tyck apps info database-explorer            # Show app details
```

### 6.5 `tyck apps init` - Scaffolding

```bash
$ tyck apps init

  ╭─────────────────────────────────────────╮
  │  Create a new Tapp                      │
  ╰─────────────────────────────────────────╯

? App ID: database-explorer
? Display name: Database Explorer  
? Description: Connect and explore databases from Tyck
? Author: Your Name <you@example.com>

? Template:
  ❯ minimal      - UI only, simplest starting point
    full         - UI + Backend (Rust WASM)
    tool-only    - Agent tool without UI
    sidebar      - Sidebar panel app

? Permissions (space to select):
  ❯ ◉ fs:read           Read files in project
    ◯ fs:write          Write files in project
    ◉ network:fetch     HTTP requests
    ◉ storage:persistent Persistent storage
    ◉ agent:tools       Register agent tools
    ◯ agent:inject      Inject context to agent
    ◯ agent:spawn       Spawn agent sessions

✔ Created database-explorer/

  Next steps:
    cd database-explorer
    tyck apps dev .

  Documentation: https://tyck.dev/docs/apps
```

#### Templates

| Template | Description | Files Generated |
|----------|-------------|-----------------|
| `minimal` | UI only, simplest | `src/app.ts`, `src/styles/app.css` |
| `full` | UI + Rust backend | + `backend/src/lib.rs`, `backend/Cargo.toml` |
| `tool-only` | Agent tool, no UI | `backend/src/lib.rs` only |
| `sidebar` | Sidebar panel | `src/app.ts` with `layout: "sidebar"` |

#### Generated Files

**manifest.json**
```json
{
  "id": "database-explorer",
  "name": "Database Explorer",
  "version": "0.1.0",
  "description": "Connect and explore databases from Tyck",
  "author": "Your Name <you@example.com>",
  "icon": "assets/icon.svg",
  
  "permissions": [
    "fs:read",
    "network:fetch",
    "storage:persistent",
    "agent:tools"
  ],
  
  "ui": {
    "entry": "dist/app.js",
    "style": "dist/app.css",
    "layout": "full"
  },
  
  "shortcuts": [
    { "key": "CmdOrCtrl+Shift+D", "action": "open" }
  ]
}
```

**src/app.ts**
```typescript
import { App, View, Panel, Text, Button, useStore } from 'tapp';
import './styles/app.css';

const app = App.create({
  name: 'database-explorer',
  
  setup(tyck) {
    const count = useStore(0);
    
    return View({ class: 'app-root' }, [
      Panel({ title: 'Database Explorer' }, [
        Text({ content: `Welcome to your new Tapp!` }),
        Text({ content: `Count: ${count.value}` }),
        Button({
          label: 'Increment',
          onClick: () => count.value++
        }),
        Button({
          label: 'Send to Agent',
          variant: 'secondary',
          onClick: async () => {
            await tyck.agent.inject('Hello from Database Explorer!');
          }
        })
      ])
    ]);
  }
});

export default app;
```

**tapp.config.ts**
```typescript
import { defineConfig } from 'tapp/config';

export default defineConfig({
  build: {
    outDir: 'dist',
    minify: true,
    sourcemap: true,
  },
  
  dev: {
    port: 5173,
    hotReload: true,
  },
  
  backend: {
    target: 'wasm32-wasip2',
    release: true,
  }
});
```

**package.json**
```json
{
  "name": "database-explorer",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "tyck apps dev .",
    "build": "tyck apps build .",
    "check": "tsc --noEmit"
  },
  "dependencies": {
    "tapp": "^0.1.0"
  },
  "devDependencies": {
    "typescript": "^5.4.0"
  }
}
```

### 6.6 Storage Layout After Install

```
~/.tyck/
├── apps/
│   ├── registry.json              # Installed apps registry
│   │   {
│   │     "apps": [
│   │       { "id": "database-explorer", "enabled": true, "installedAt": "..." }
│   │     ]
│   │   }
│   └── database-explorer/         # App directory
│       ├── manifest.json
│       ├── dist/
│       │   ├── app.js
│       │   ├── app.css
│       │   └── backend.wasm
│       ├── assets/
│       │   └── icon.svg
│       └── data/                  # App's persistent storage
│           ├── store.json         # JSON key-value store
│           └── app.db             # SQLite database
├── settings.json
├── themes/
└── ...
```

---

## 7. App Views (UI Integration)

### 7.1 Layout Modes

| Layout | Description | Zones Affected |
|--------|-------------|----------------|
| `full` | App replaces ContextZone + FocusZone | InsightZone remains |
| `sidebar` | App replaces ContextZone only | FocusZone + InsightZone remain |
| `panel` | App in bottom panel (like Terminal) | All zones remain |
| `modal` | Overlay dialog | All zones remain (dimmed) |

### 7.2 Full Layout Mode

```
┌─────────────────────────────────────────────────────────────────────┐
│ AwarenessBar [App: Database Explorer]              AI status        │
├─────────────────────────────────────────────┬───────────────────────┤
│                                              │ InsightZone           │
│           App Webview Content                │ (Agent terminal)      │
│           (Database Explorer UI)             │ - Always visible      │
│                                              │ - User can interact   │
│                                              │                       │
├─────────────────────────────────────────────┴───────────────────────┤
│ CommandRail                                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.3 Sidebar Layout Mode

```
┌─────────────────────────────────────────────────────────────────────┐
│ AwarenessBar                                       AI status        │
├──────────────┬──────────────────────────┬───────────────────────────┤
│ App Webview  │ FocusZone                │ InsightZone               │
│ (Sidebar)    │ (Monaco editor)          │ (Agent terminal)          │
│              │                          │                           │
│              │                          │                           │
├──────────────┴──────────────────────────┴───────────────────────────┤
│ CommandRail                                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.4 App Switching

Apps accessible via:
- **Keyboard**: `Cmd+Shift+A` opens app launcher
- **AwarenessBar**: App icon/dropdown
- **App-specific shortcuts**: Defined in manifest

```typescript
// App launcher shows installed apps
[
  { id: 'database-explorer', name: 'Database Explorer', icon: '...', shortcut: 'Cmd+Shift+D' },
  { id: 'api-tester', name: 'API Tester', icon: '...', shortcut: 'Cmd+Shift+T' },
  ...
]
```

---

## 8. Security

### 8.1 Webview Isolation

Each app runs in an isolated Tauri webview:

- **CSP (Content Security Policy)**: Restricts scripts, styles, connections
- **Origin isolation**: Apps can't access each other's storage or DOM
- **No direct filesystem**: All I/O through Tapp API
- **Capability-based**: Only granted permissions are available

### 8.2 Permission Enforcement

```typescript
// Runtime permission checks
class TappRuntime {
  private permissions: Set<string>;
  
  async readFile(path: string): Promise<string> {
    if (!this.permissions.has('fs:read')) {
      throw new TappPermissionError('fs:read permission required');
    }
    // Proceed with read
  }
  
  async spawnAgentSession(options: SpawnOptions): Promise<AgentSession> {
    if (!this.permissions.has('agent:spawn')) {
      throw new TappPermissionError('agent:spawn permission required');
    }
    // Proceed with spawn
  }
}
```

### 8.3 Network Restrictions

For `network:fetch` (not `network:unrestricted`):

```json
{
  "permissions": ["network:fetch"],
  "network": {
    "allowedOrigins": [
      "https://api.example.com",
      "https://*.database-service.com"
    ]
  }
}
```

### 8.4 WASM Sandboxing

Backend WASM modules run with WASI capability-based security:

- **No ambient filesystem access**: Must be granted specific paths
- **No ambient network**: Must be granted specific capabilities
- **Memory isolation**: Each module has isolated linear memory
- **Resource limits**: CPU time, memory caps

---

## 9. Full API Reference

### 9.1 TyckAppAPI

```typescript
interface TyckAppAPI {
  // ─── Lifecycle ───
  ready(): Promise<void>;
  close(): void;
  
  // ─── Editor ───
  editor: {
    getActiveFile(): Promise<FileInfo | null>;
    getOpenFiles(): Promise<FileInfo[]>;
    getProjectRoot(): Promise<string>;
    getSelection(): Promise<Selection | null>;
    getCursor(): Promise<Position>;
    openFile(path: string, options?: OpenOptions): Promise<void>;
    revealLine(path: string, line: number): Promise<void>;
    
    onFileOpen(callback: (file: FileInfo) => void): Unsubscribe;
    onFileClose(callback: (file: FileInfo) => void): Unsubscribe;
    onFileSave(callback: (file: FileInfo) => void): Unsubscribe;
    onSelectionChange(callback: (selection: Selection | null) => void): Unsubscribe;
    onActiveFileChange(callback: (file: FileInfo | null) => void): Unsubscribe;
  };
  
  // ─── Agent ───
  agent: {
    // Active session
    getActiveSession(): Promise<AgentSession>;
    inject(text: string): Promise<void>;
    injectFile(path: string, options?: InjectOptions): Promise<void>;
    injectCodeBlock(code: string, language: string): Promise<void>;
    
    // Session management
    spawn(options: SpawnOptions): Promise<AgentSession>;
    getSessions(): Promise<AgentSession[]>;
    
    // Events
    onOutput(callback: (chunk: string, session: AgentSession) => void): Unsubscribe;
    onStatusChange(callback: (status: AgentStatus) => void): Unsubscribe;
  };
  
  // ─── Tools ───
  tools: {
    register(definition: ToolDefinition): void;
    unregister(name: string): void;
    list(): ToolDefinition[];
    exportAsMcp(options: McpExportOptions): void;
  };
  
  // ─── Hooks ───
  hooks: {
    onBeforeInput(callback: InputHook): Unsubscribe;
    onAfterOutput(callback: OutputHook): Unsubscribe;
    onToolCall(callback: ToolCallHook): Unsubscribe;
    onSessionStart(callback: SessionHook): Unsubscribe;
    onSessionEnd(callback: SessionHook): Unsubscribe;
  };
  
  // ─── Storage ───
  storage: {
    session(): SessionStore;
    json(): Promise<JsonStore>;
    sqlite(): Promise<SqliteStore>;
  };
  
  // ─── Filesystem ───
  fs: {
    readFile(path: string): Promise<string>;
    readFileBytes(path: string): Promise<Uint8Array>;
    writeFile(path: string, content: string): Promise<void>;
    writeFileBytes(path: string, content: Uint8Array): Promise<void>;
    readDirectory(path: string): Promise<DirectoryEntry[]>;
    exists(path: string): Promise<boolean>;
    stat(path: string): Promise<FileStat>;
    mkdir(path: string, options?: MkdirOptions): Promise<void>;
    remove(path: string, options?: RemoveOptions): Promise<void>;
    rename(from: string, to: string): Promise<void>;
    glob(pattern: string): Promise<string[]>;
    
    onFileChange(pattern: string, callback: (event: FSEvent) => void): Unsubscribe;
  };
  
  // ─── UI ───
  ui: {
    showNotification(options: NotificationOptions): void;
    showDialog(options: DialogOptions): Promise<DialogResult>;
    showQuickPick<T>(options: QuickPickOptions<T>): Promise<T | null>;
    showInputBox(options: InputBoxOptions): Promise<string | null>;
    setTitle(title: string): void;
    setBadge(count: number | null): void;
    setStatus(status: string | null): void;
  };
  
  // ─── Git ───
  git: {
    isRepo(): Promise<boolean>;
    getStatus(): Promise<GitStatus>;
    getBranch(): Promise<string>;
    getBranches(): Promise<Branch[]>;
    getChanges(): Promise<FileChange[]>;
    getLog(options?: LogOptions): Promise<Commit[]>;
    
    onStatusChange(callback: (status: GitStatus) => void): Unsubscribe;
    onBranchChange(callback: (branch: string) => void): Unsubscribe;
  };
  
  // ─── Messaging (v2) ───
  messaging: {
    broadcast(channel: string, data: any): void;
    onBroadcast(channel: string, callback: (data: any, from: string) => void): Unsubscribe;
    sendTo(appId: string, message: any): Promise<any>;
    onMessage(callback: (message: any, from: string) => void): Unsubscribe;
  };
}
```

### 9.2 Type Definitions

```typescript
interface FileInfo {
  path: string;
  name: string;
  isDirectory: boolean;
  size: number;
  modified: Date;
}

interface Selection {
  start: Position;
  end: Position;
  text: string;
}

interface Position {
  line: number;
  column: number;
}

interface AgentSession {
  id: string;
  name: string;
  provider: string;
  status: 'idle' | 'running' | 'exited';
  visible: boolean;
  createdAt: Date;
  
  send(text: string): Promise<void>;
  onOutput(callback: (chunk: string) => void): Unsubscribe;
  waitForIdle(): Promise<void>;
  getTranscript(): Promise<Message[]>;
  show(): Promise<void>;
  hide(): Promise<void>;
  focus(): Promise<void>;
  kill(): Promise<void>;
}

interface ToolDefinition {
  name: string;
  description: string;
  parameters: JsonSchema;
  handler: (args: any) => Promise<any>;
}

interface SpawnOptions {
  provider?: string;
  systemPrompt?: string;
  cwd?: string;
  visible?: boolean;
  env?: Record<string, string>;
  name?: string;
}

type Unsubscribe = () => void;
```

---

## 10. Implementation Phases

### Phase 1: Foundation (MVP)

1. **Tapp Framework Core**
   - Basic component library (View, Panel, Text, Button, List)
   - Reactivity system (useStore, useEffect)
   - Build tooling (tapp build, tapp dev)

2. **App Container**
   - Webview-based app host
   - Message passing bridge
   - Permission checking

3. **Basic API**
   - `editor` API (read-only)
   - `agent.inject()` 
   - `storage.session()` and `storage.json()`

4. **CLI**
   - `tyck apps init`
   - `tyck apps dev`
   - `tyck apps build`
   - `tyck apps install`

### Phase 2: Agent Integration

1. **Agent Sessions**
   - `agent.spawn()` implementation
   - Session management
   - Output streaming

2. **Tapp Tools**
   - Tool registration
   - Execution routing
   - Result passing

3. **Agent Hooks**
   - Input/output interception
   - Session lifecycle events

4. **SQLite Storage**
   - Add tauri-plugin-sql
   - Per-app database isolation

### Phase 3: Polish & Ecosystem

1. **Additional Layouts**
   - Sidebar mode
   - Panel mode  
   - Modal dialogs

2. **Advanced Components**
   - VirtualList, DataGrid, Tree
   - Rich text editor
   - Charts/visualizations

3. **WASM Backend**
   - tapp-backend crate
   - WASI sandboxing
   - Hot reload for Rust

4. **App Store (v2)**
   - Remote registry
   - Package signing
   - Auto-updates

---

## 11. Comparison with Existing Systems

| Feature | VS Code | Cursor | Zed | Tapp |
|---------|---------|--------|-----|------|
| **UI Technology** | Webview (free) | Webview + Native | WASM | Webview (constrained) |
| **Framework** | Any | Any | WASM/Rust | Tapp Framework |
| **Isolation** | Process | Mixed | WASM sandbox | Webview sandbox |
| **Agent Integration** | N/A | MCP + Hooks | MCP | Tools + Hooks + Spawn |
| **Storage** | ExtensionContext | Similar | Key-value | Session + JSON + SQLite |
| **Tool Protocol** | N/A | MCP | MCP | Tapp Tools (+ MCP export) |
| **Backend** | Node.js | Node.js | WASM | WASM (Rust) |

### Key Differentiators

1. **Constrained Framework**: Consistent UX, guaranteed performance, smaller bundles
2. **Agent-First**: Spawn sessions, register tools, hook I/O - all native
3. **Tapp Tools**: Simpler than MCP for internal use, optional MCP export
4. **InsightZone Preserved**: Agent always visible regardless of app
5. **Multi-Storage**: Session (ephemeral) + JSON (simple) + SQLite (powerful)

---

## 12. Open Questions (Resolved)

| Question | Decision |
|----------|----------|
| WASM backend in v1? | No, JS-only for v1, WASM in v2 |
| Hot reload? | Yes, essential for DX |
| App theming? | Inherit Tyck CSS vars, allow overrides |
| Background apps? | Yes, useful for tools/automation |
| Multi-window? | Not in v1 |
| UI freedom? | No, constrained Tapp Framework |
| Tool protocol? | Tapp Tools native, optional MCP export |
| Agent session spawning? | Yes, core feature |

---

## 13. Next Steps

1. **Finalize API contracts** - Lock down TypeScript interfaces
2. **Build Tapp Framework** - Core components + reactivity
3. **Prototype app container** - Webview host + message bridge
4. **Build CLI tooling** - init, dev, build, install
5. **Sample app** - Database Explorer to validate design
6. **Documentation site** - https://tyck.dev/docs/apps

---

## Appendix A: Sample App Ideas

1. **Database Explorer**: Connect to databases, browse tables, run queries, agent can query
2. **API Tester**: Postman-like HTTP client, agent can make requests
3. **Documentation Browser**: Render and search project docs
4. **Dependency Analyzer**: Visualize and manage dependencies
5. **Performance Profiler**: Profile and analyze code performance
6. **Git Graph**: Visual git history and branch management
7. **Snippet Manager**: Organize and inject code snippets
8. **AI Prompt Library**: Manage and use prompt templates
9. **Test Runner**: Run tests, show results, agent can run specific tests
10. **Deployment Manager**: Deploy to various platforms, agent can trigger deploys

## Appendix B: Prior Art References

- [VS Code Extension API](https://code.visualstudio.com/api)
- [Cursor Plugin System](https://cursor.com/docs/reference/plugins)
- [Zed Extension API](https://docs.rs/zed_extension_api)
- [Tauri Plugin Development](https://tauri.app/develop/plugins)
- [Model Context Protocol](https://modelcontextprotocol.io)
- [vscode-messenger](https://www.typefox.io/blog/vs-code-messenger/)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
