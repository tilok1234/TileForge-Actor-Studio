# Architecture

TileForge Actor Studio uses a small layered design.

```text
Codex ─┐
Claude ├── MCP gateway ── TypeScript adapter ─┐
Antigravity ┘                                 ├── .studio session protocol
Svelte UI ── Tauri commands ── Rust adapter ──┘
```

## Layers

### Contract

`contracts/tileforge-actor-32-v1.json` is the portable, versioned definition of
frame, art, animation, and approval rules. The TypeScript representation in
`src/lib/studio/contract.ts` must stay equivalent.

### Shared studio core

`src/lib/studio/` contains client-neutral types, session creation, and prompt
compilation. Business rules belong here.

### MCP gateway

`mcp/` exposes the shared core over standard MCP transports. It owns no art
style rules and grants no final-approval capability. Its session adapter reads
and publishes the same documents as the desktop backend.

### Desktop shell

`src/` renders the focused workflow. Tauri 2 in `src-tauri/` supplies the native
Windows shell and the desktop adapter for durable session commands. The UI is
the eventual human approval surface.

### Local workspace

`.studio/` is the shared persistence boundary. Both adapters use
`.studio/sessions/<session-id>/session.json` and a sibling `candidates/`
directory. A complete session is first written to a hidden temporary directory,
then published with one same-parent rename so readers never observe a partial
record. `TFAS_WORKSPACE` redirects the root for either adapter and for tests.

The JSON document, identity rules, brief limits, and directory layout form one
local protocol; the Rust and TypeScript adapters are not separate stores.
`tests/fixtures/session-v1.json` is read by both test suites to detect drift.
`.studio/` is not source-controlled.

## Reference boundary

TileForge remains a separate project. Actor Studio will eventually consume a
pinned, copied reference pack with provenance; it will not import arbitrary
runtime code or write back into TileForge. The old animation editor is design
evidence only, not a dependency.
