# TileForge Actor Studio

A narrow desktop workflow for creating **32 px TileForge mobs and NPCs with AI**.
The artist supplies the identity; the studio supplies the boundaries.

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
cargo check --manifest-path src-tauri/Cargo.toml
```

## Architecture

- `src/` — Svelte UI and shared studio domain code
- `src-tauri/` — native Tauri 2 shell
- `contracts/` — versioned machine-readable world contract
- `mcp/` — client-neutral MCP gateway
- `.studio/` — ignored local sessions and generated artifacts
- `docs/` — architecture and agent workflow

The current milestone establishes the shell, contract, prompt compiler, local
session format, and MCP surface. Image generation, validators, reference-pack
ingestion, and export assembly are the next implementation layers.
