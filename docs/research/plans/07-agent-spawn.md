# Plan 07: Agent Session Spawning

## Problem

The design doc's marquee feature — spawning background AI agent sessions from apps — is completely unimplemented. Only session metadata tracking exists.

## Goal

Apps with `agent:spawn` permission can spawn new agent sessions (e.g., Claude Code), send input, receive output, and manage session lifecycle.

## Prerequisites

- Plan 01 (WIT Host Imports)
- Plan 06 (Agent Inject — shares the `host-agent` interface)

## Scope

- Extend `host-agent` WIT interface with spawn/send/read/kill
- Implement process management on the host side
- Implement output streaming/polling from WASM
- Add guest-side API

## Design

```wit
interface host-agent {
    // ... (inject from Plan 06)

    /// Spawn a new agent session. Returns session ID.
    spawn-session: func(options: string) -> result<string, string>;

    /// Send text input to a session
    send-to-session: func(session-id: string, text: string) -> result<_, string>;

    /// Read available output from a session (non-blocking, returns what's available)
    read-session-output: func(session-id: string) -> result<string, string>;

    /// Get session status: "idle" | "running" | "exited"
    get-session-status: func(session-id: string) -> result<string, string>;

    /// Kill a session
    kill-session: func(session-id: string) -> result<_, string>;
}
```

## Implementation Steps

1. Extend `host-agent` in `tapp.wit` with spawn/send/read/kill
2. Implement `AgentProcessManager` in `src-tauri/src/apps/`:
   - Uses `portable-pty` (already a dependency) to spawn agent processes
   - Manages stdin/stdout/stderr for each session
   - Output buffered in a ring buffer per session
   - Tracks process state (running/idle/exited)
3. Implement host handlers:
   - `spawn-session`: parse options JSON, spawn process, register in AgentBridge, return ID
   - `send-to-session`: write to session's stdin
   - `read-session-output`: drain output buffer, return as string
   - `get-session-status`: check process state
   - `kill-session`: send SIGTERM, cleanup
4. Update guest SDK `agent.rs`:
   - `AgentSession::spawn(options) -> Result<AgentSession>`
   - `session.send(text) -> Result<()>`
   - `session.read_output() -> Result<String>`
   - `session.status() -> Result<SessionStatus>`
   - `session.kill() -> Result<()>`
5. Permission checks: require `agent:spawn`
6. Resource limits: max 3 concurrent sessions per app
7. Test: app that spawns a session, sends "echo hello", reads back output

## Challenges

- **Output streaming**: WASM is synchronous. The guest must poll for output. The host buffers and the guest calls `read_session_output` on each render or action cycle.
- **Process lifecycle**: Must handle orphan cleanup when apps crash.
- **Provider selection**: Initially hardcode to system shell. Claude Code integration comes later.

## Files Modified

- `packages/tapp/wit/tapp.wit`
- `src-tauri/src/wasm/instance.rs`
- `src-tauri/src/apps/agent_bridge.rs` (major expansion)
- `src-tauri/src/apps/agent_process.rs` (new)
- `packages/tapp/src/agent.rs`

## Estimated Complexity

High. This is the largest single feature. Process management + output buffering + cleanup is substantial.
