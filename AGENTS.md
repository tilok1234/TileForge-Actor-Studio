# Agent Guide

Read `HANDOFF.md`, `docs/AGENT_WORKFLOW.md`, `docs/ROADMAP.md`, `docs/MCP.md`,
and `contracts/tileforge-actor-32-v1.json` before doing sprite work.

## Mission

Keep TileForge Actor Studio small and coherent: one 32 px mob or NPC, from a
creative brief to an approved four-direction walk-cycle export.

## Hard boundaries

- Do not edit the external `animation_editor_live` repository.
- Do not edit the external `Semantic tile generator design` repository.
- Treat external TileForge material as read-only evidence.
- Never let an agent mark art as finally approved.
- Never overwrite a generated candidate; create a new immutable revision.
- Do not expand version 1 into tiles, maps, bosses, effects, equipment,
  paperdolls, batch generation, or publishing.
- Do not write secrets or client-specific credentials into the repository.

## Required behavior

- Use `contracts/tileforge-actor-32-v1.json` as the source of truth.
- Keep shared domain behavior in `src/lib/studio/`, not inside one AI client.
- Expose agent operations through `mcp/` so Codex, Claude, and Antigravity get
  equivalent capabilities.
- Store local work under `.studio/`; it is intentionally ignored by Git.
- Keep the human approval boundary visible in both the UI and tool descriptions.
- Treat a generation request as a durable handoff, not a dispatched AI job.
- Run `npm run check`, `npm run build`, `npm run test:mcp`, and the relevant
  Rust check before claiming a milestone is complete.
- Keep `HANDOFF.md`, `docs/ROADMAP.md`, and `docs/DECISIONS.md` synchronized
  with implemented behavior.

## Change discipline

Prefer a small vertical slice over a wide framework. If a feature does not move
one actor through Brief → Concept → Turnaround → Animate → World test → Export,
it probably does not belong in version 1.
