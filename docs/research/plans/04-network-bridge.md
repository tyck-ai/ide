# Plan 04: Network Bridge — HTTP Requests from WASM

## Problem

The host `NetworkManager` supports HTTP/HTTPS requests with host allowlists, but WASM apps can't call it. There's no WASM→host bridge for network operations.

## Goal

Apps with `network:fetch` permission can make HTTP/HTTPS requests through a host-imported API, subject to the allowlist in their manifest.

## Prerequisites

Plan 01 (WIT Host Imports).

## Scope

- Add `host-network` interface to WIT
- Implement host-side network handler
- Add guest-side `Net` or `Http` API to the SDK

## Design

```wit
interface host-network {
    /// Make an HTTP request. Headers and body are JSON-encoded.
    http-request: func(method: string, url: string, headers: string, body: option<list<u8>>) -> result<string, string>;
}
```

Response is JSON: `{ "status": 200, "headers": [...], "body": "base64..." }`

Keep it simple — one function for all HTTP methods. The host handles TLS, allowlist checking, and connection management.

## Implementation Steps

1. Add `host-network` to `tapp.wit`
2. Implement host handler:
   - Look up app's manifest to get `network.allowed_hosts`
   - Create a `NetworkManager` per request (or reuse from app state)
   - Call `NetworkManager::http_request()`
   - Return response as JSON
3. Add `packages/tapp/src/http.rs` guest-side:
   - `pub fn get(url: &str) -> Result<Response>`
   - `pub fn post(url: &str, body: &[u8]) -> Result<Response>`
   - `pub fn request(method: &str, url: &str, headers: &[(&str, &str)], body: Option<&[u8]>) -> Result<Response>`
   - `Response` struct with `status`, `headers`, `body`
4. Export from lib.rs
5. Test: app that fetches a public JSON API and displays results

## Security

- Always check `network:fetch` or `network:unrestricted` permission
- Allowlist enforced via existing `NetworkManager::is_host_allowed()`
- TLS used for HTTPS (already implemented)
- No raw socket access from WASM (only HTTP)

## Files Modified

- `packages/tapp/wit/tapp.wit`
- `src-tauri/src/wasm/instance.rs`
- `packages/tapp/src/http.rs` (new)
- `packages/tapp/src/lib.rs`

## Estimated Complexity

Low-Medium. The host-side HTTP code already works — this is routing it through WIT.
