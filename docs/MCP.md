# MCP setup

Actor Studio exposes one client-neutral MCP server. Start it from the repository
root:

```powershell
npm install
npm run mcp:http
```

The server listens only on localhost:

```text
http://127.0.0.1:7331/mcp
```

The shared HTTP transport is the preferred setup because Codex, Claude Code,
and Antigravity can use the same running gateway.

## Codex

Add the local endpoint:

```powershell
codex mcp add tileforge-actor-studio --url http://127.0.0.1:7331/mcp
codex mcp list
```

An equivalent TOML example is in `.codex/config.toml.example`. Do not copy it
into a user-level configuration without reviewing the target first.

## Claude Code

This repository includes a project-scoped `.mcp.json`. Claude Code will ask for
trust before using it. You can also add it explicitly:

```powershell
claude mcp add --transport http --scope project tileforge-actor-studio http://127.0.0.1:7331/mcp
claude mcp get tileforge-actor-studio
```

## Antigravity

Open **Manage MCP Servers → View raw config** and add this entry under
`mcpServers` in the shared Gemini/Antigravity MCP configuration:

```json
{
  "mcpServers": {
    "tileforge-actor-studio": {
      "serverUrl": "http://127.0.0.1:7331/mcp"
    }
  }
}
```

Antigravity 2.0 documents its shared configuration at
`~/.gemini/config/mcp_config.json`. Use the UI to locate the live file rather
than assuming a path on older installations.

## Local stdio fallback

For a client that cannot reach the localhost HTTP endpoint, launch:

```text
node --import tsx mcp/src/index.ts --transport stdio
```

Set the process working directory to the repository root. HTTP is preferred for
this project so every client observes the same gateway behavior.

## Available tools

- `get_studio_contract`
- `compile_actor_prompt`
- `create_sprite_session`
- `list_sprite_sessions`
- `get_sprite_session`
- `import_concept_candidate`
- `list_concept_candidates`
- `get_concept_candidate`
- `validate_concept_candidate`
- `create_turnaround_candidate`
- `list_turnaround_candidates`
- `get_turnaround_candidate`
- `validate_turnaround_candidate`
- `create_walk_cycle_candidate`
- `list_walk_cycle_candidates`
- `get_walk_cycle_candidate`
- `validate_walk_cycle_candidate`
- `create_world_test_candidate`
- `list_world_test_candidates`
- `get_world_test_candidate`
- `validate_world_test_candidate`

The gateway also exposes:

- resource: `studio://contracts/tileforge-actor-32-v1`
- prompt: `design_tileforge_actor`

No tool can approve final art.

`import_concept_candidate` accepts PNG bytes with imported or generated
provenance. Generated provenance must name its provider, but Actor Studio does
not integrate or select a provider in M02. Successful intake records structural
evidence and `unreviewed` status; it never implies visual acceptance.

`validate_concept_candidate` is read-only and local. It returns seven ordered
contract results plus Pass/Fail/Not assessed totals tied to the candidate
SHA-256. Ground luma is Not assessed until World Test supplies a pinned ground
and placement.
The report's visual judgment is also Not assessed and user-owned; no MCP tool
can change that state.

`create_turnaround_candidate` is available only after the user explicitly
selects a Concept. Its down PNG must preserve that exact Concept's bytes. It
atomically publishes down, right, up, and left PNGs plus a versioned document;
it cannot accept identity consistency or approve final art.

`validate_turnaround_candidate` rehashes all four immutable sources and
recomputes the structural report for each direction. Its aggregate report keeps
identity consistency Not assessed with user authority.

`create_walk_cycle_candidate` is available only after the user explicitly
accepts a Turnaround. It requires four frames per canonical direction at 300
ms; frame 0 must preserve each accepted Turnaround view exactly. It atomically
publishes sixteen PNGs plus a versioned document and cannot accept motion,
approve final art, or publish.

`validate_walk_cycle_candidate` rehashes all sixteen immutable frame sources
and recomputes structural evidence per frame. Its aggregate report keeps motion
and readability Not assessed with user authority.

`create_world_test_candidate` is available only after the user explicitly
accepts Walk Cycle motion/readability. It records the exact sixteen-frame
receipt, verifies the copied reference pack, and atomically prepares four
scenes across four themes with the local deterministic compositor. It uses no
AI service and cannot approve final art.

`validate_world_test_candidate` rehashes all sixteen immutable previews and
recomputes 256 frame-to-ground luma comparisons against the pinned pack. Its
final-art judgment remains Not assessed with user authority.

## Shared desktop state

MCP session tools and the Tauri desktop use the same local workspace:
`.studio/sessions` by default, or `TFAS_WORKSPACE` when redirected. Session
directories are published atomically and are never overwritten. A session
created from the desktop can therefore be listed and read through MCP without a
conversion or copy step. Concept candidate directories use the same rule and
preserve the exact original `source.png`; Turnaround directories preserve
`turnaround.json` plus `down.png`, `right.png`, `up.png`, and `left.png`.
Walk Cycle directories preserve `walk-cycle.json` plus four numbered frame
PNGs for every canonical direction. World Test directories preserve
`world-test.json` plus sixteen scene/theme preview PNGs. Either adapter can
list and read an artifact created by the other.

Do not configure an AI provider that incurs incremental charges. Any future AI
connection must be covered by the user's existing subscriptions.
