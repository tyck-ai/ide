# Plan 06: Agent Context Injection

## Problem

Apps can't inject context into the active agent session. The design doc's `tyck.agent.inject()` is unimplemented.

## Goal

Apps with `agent:inject` permission can inject text, file references, or code blocks into the active agent session's input.

## Prerequisites

Plan 01 (WIT Host Imports).

## Scope

- Add `host-agent` interface to WIT (inject portion)
- Implement host-side injection via the agent terminal
- Add guest-side `Agent` API to the SDK

## Design

```wit
interface host-agent {
    /// Inject text into the active agent session's input
    inject-text: func(text: string) -> result<_, string>;

    /// Inject a file reference with optional selection
    inject-file: func(path: string, options: string) -> result<_, string>;
}
```

## Implementation Steps

1. Add `inject-text` and `inject-file` to `host-agent` in `tapp.wit`
2. Implement host handler:
   - Find the active agent session (from the terminal/agent store)
   - Append the injected text to the session's input
   - Use Tauri events to notify the frontend to update the input field
3. Add `packages/tapp/src/agent.rs` guest-side:
   - Replace stub `inject_to_active()` with WIT import call
   - `pub fn inject(text: &str) -> Result<()>`
   - `pub fn inject_file(path: &str) -> Result<()>`
4. Check `agent:inject` permission before allowing injection
5. Test: app with a button that injects "Hello from MyApp!" into the agent input

## Security

- Injected text is treated as user input (not system prompt)
- Rate limiting: max 10 injections per second per app
- Size limit: max 100KB per injection
- Permission check on every call

## Files Modified

- `packages/tapp/wit/tapp.wit`
- `src-tauri/src/wasm/instance.rs`
- `src-tauri/src/apps/agent_bridge.rs`
- `packages/tapp/src/agent.rs`

## Estimated Complexity

Medium. The injection itself is simple; the challenge is getting text into the active terminal session from the backend.
