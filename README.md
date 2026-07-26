# TileForge Actor Studio

A narrow desktop workflow for creating **32 px TileForge mobs and NPCs with AI**.
The artist supplies the identity; the studio supplies the boundaries.

Current status: **M02 immutable Concept candidates complete; M03 contract
validation is next.** The desktop and MCP gateway share durable local sessions
and never-overwritten concept PNG revisions. No image-generation provider is
selected or integrated.

The initial workflow has six deliberate stages:

1. Brief
2. Concept
3. Turnaround
4. Animate
5. World test
6. Export

This repository is a clean-room successor to a larger experimental animation
editor. It does not modify or depend on that editor. TileForge is also treated
as a read-only visual reference: Actor Studio may consume a future pinned
reference pack, but it never writes into the TileForge repository.

## Documentation

- [HANDOFF.md](HANDOFF.md) — verified continuation point and implementation truth
- [docs/ROADMAP.md](docs/ROADMAP.md) — milestone sequence and acceptance criteria
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — system boundaries and layers
- [docs/DECISIONS.md](docs/DECISIONS.md) — durable design decisions and open ADR
- [docs/AGENT_WORKFLOW.md](docs/AGENT_WORKFLOW.md) — authority and approval flow
- [docs/MCP.md](docs/MCP.md) — Codex, Claude Code, and Antigravity setup
- [docs/START_NEW_CHAT.md](docs/START_NEW_CHAT.md) — copy-paste continuation prompt

## What version 1 is

- One mob or NPC at a time
- A locked 32 × 32 px world contract
- Four-direction, four-frame walk cycles
- Immutable generated candidates
- Explicit human approval before final art
- One MCP gateway shared by Codex, Claude Code, and Antigravity

## What version 1 is not

- A map or tile editor
- A general animation suite
- A paperdoll or equipment system
- A boss/effects pipeline
- A batch asset factory
- An autonomous approval or publishing system

## Start the desktop UI

Requirements: Node.js 20+, Rust, and the platform prerequisites for Tauri 2.

```powershell
npm install
npm run tauri dev
```

For a browser-only UI pass:

```powershell
npm run dev
```

## Start the agent gateway

Run one local Streamable HTTP server:

```powershell
npm run mcp:http
```

It listens only on `http://127.0.0.1:7331/mcp`. All supported AI clients can
connect to that same endpoint. See [docs/MCP.md](docs/MCP.md).

## Verify

```powershell
npm run check
npm run build
npm run test:mcp
npm run test:mcp:stdio
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm audit --audit-level=moderate
```

For the HTTP transport smoke test, run `npm run mcp:http` in one terminal and
`npm run test:mcp:http` in another.

## Architecture

- `src/` — Svelte UI and shared studio domain code
- `src-tauri/` — native Tauri 2 shell
- `contracts/` — versioned machine-readable world contract
- `mcp/` — client-neutral MCP gateway
- `.studio/` — ignored local sessions and generated artifacts
- `docs/` — architecture and agent workflow

The completed M02 milestone extends the shared `.studio` protocol with
immutable candidate documents and original PNG bytes. The desktop restores,
lists, compares, and zooms saved revisions, while MCP exposes equivalent
import/list/read operations. Intake evidence is structural only; deterministic
contract validation and human visual approval remain later, separate gates.
