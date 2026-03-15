# Plan 09: TypeScript SDK — Lower the Barrier to Entry

## Problem

The current SDK requires Rust. Most IDE extension developers know TypeScript, not Rust. The design doc envisioned a TypeScript API. This is the biggest adoption barrier.

## Goal

Developers can write tapp extensions in TypeScript that compile to WASM and run in the same runtime as Rust apps.

## Approach Options

### Option A: AssemblyScript (TS subset → WASM)

- **Pros**: TypeScript-like syntax, compiles directly to WASM, small runtime
- **Cons**: Not full TypeScript (no `any`, limited stdlib), different toolchain
- **Effort**: Medium — need AS versions of SDK types

### Option B: componentize-js (JS → WASM Component)

- **Pros**: Real JavaScript/TypeScript, uses the standard component model
- **Cons**: Larger runtime (~2MB StarlingMonkey engine), slower startup
- **Effort**: Low-Medium — the `componentize-js` tool from Bytecode Alliance handles compilation

### Option C: Embedded JS Runtime (QuickJS/Boa in host)

- **Pros**: Real JS, no WASM compilation needed, fastest dev loop
- **Cons**: Different runtime model, not WASM-sandboxed, security concerns
- **Effort**: High — new runtime alongside wasmtime

### Recommendation: Option B (componentize-js)

It's the official Bytecode Alliance approach for JS→WASM components. It uses the same WIT interface, same component model, same host imports. The SDK would be a thin TypeScript wrapper around the WIT bindings.

## Implementation Steps

1. **Create `packages/tapp-ts/` package**
   - TypeScript types mirroring the Rust SDK
   - `App`, `UINode`, `Action`, `Response`, `ToolResult`, etc.
   - Builder functions: `ui.text()`, `ui.button()`, `ui.vstack()`, etc.

2. **Create `packages/tapp-ts/src/index.ts`**
   - Export the same API surface as the Rust SDK
   - Implement WIT import/export bindings via `@bytecodealliance/jco`

3. **Update CLI `tapp init --lang ts`**
   - New template that generates a TypeScript project
   - `package.json` with `tapp-ts` dependency
   - `src/app.ts` with the TypeScript API

4. **Update CLI `tapp build` for TS projects**
   - Detect TypeScript project (no `Cargo.toml`, has `package.json`)
   - Run `componentize-js` to compile to WASM component
   - Output same `.wasm` format that the host expects

5. **Test: TypeScript version of minimal-app**
   ```typescript
   import { App, ui, Context, Action, Response } from 'tapp-ts';

   let counter = 0;

   export const app: App = {
     init(ctx: Context) { },
     shutdown() { },
     handle(action: Action): Response {
       if (action.name === 'increment') counter++;
       return Response.render();
     },
     render() {
       return ui.vstack([
         ui.text(`Count: ${counter}`),
         ui.button('+').onClick('increment'),
       ]);
     },
   };
   ```

## Prerequisites

- Plan 01 (WIT Host Imports) — the TS SDK needs the same WIT interface
- The WIT interface must be stable before building a second SDK on it

## Files Created

- `packages/tapp-ts/` (new package)
- `packages/tapp-ts/src/index.ts`
- `packages/tapp-ts/src/ui.ts`
- `packages/tapp-ts/src/types.ts`
- `packages/tapp-ts/package.json`
- `packages/tapp-ts/tsconfig.json`
- `packages/tapp-cli/src/templates_ts.rs` (TS templates)

## Estimated Complexity

High. New SDK, new build pipeline, new templates. But the runtime is shared.
