# Architecture

TileForge Actor Studio uses a small layered design.

```text
Codex ─┐
Claude ├── MCP gateway ── shared contract/session core ── local .studio data
Antigravity ┘                         │
                                     └── Tauri 2 + Svelte approval UI
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
style rules and grants no final-approval capability.

### Desktop shell

`src/` renders the focused workflow. Tauri 2 in `src-tauri/` supplies the native
Windows shell. The UI is the eventual human approval surface.

### Local workspace

`.studio/` stores sessions and, later, immutable generated candidates,
validation reports, and draft exports. It is not source-controlled.

## Reference boundary

TileForge remains a separate project. Actor Studio will eventually consume a
pinned, copied reference pack with provenance; it will not import arbitrary
runtime code or write back into TileForge. The old animation editor is design
evidence only, not a dependency.
