# TileForge Actor Studio Handoff

This is the canonical continuation point for a new agent or chat.

## Start here

Work only in:

```text
C:\Users\headc\Documents\TileForge-Actor-Studio
```

Before editing, run:

```powershell
Get-Location
git status -sb
git log -3 --oneline --decorate
```

Then read, in order:

1. `AGENTS.md`
2. this handoff
3. `README.md`
4. `contracts/tileforge-actor-32-v1.json`
5. `docs/ARCHITECTURE.md`
6. `docs/AGENT_WORKFLOW.md`
7. `docs/ROADMAP.md`
8. `docs/DECISIONS.md`

Do not assume the handoff is newer than Git. Reconcile it with the live branch,
working tree, and tests.

## Product intent

TileForge Actor Studio is a deliberately narrow AI-assisted desktop workflow
for creating one 32 px TileForge mob or NPC at a time:

```text
Brief -> Concept -> Turnaround -> Animate -> World test -> Export
```

The artist describes identity and mood. The versioned contract supplies the
world boundaries. Agents may create, compare, validate, and prepare candidates,
but only the user may approve final art or publishing.

## Repository and boundaries

- Remote: `https://github.com/tilok1234/TileForge-Actor-Studio`
- Default branch: `main`
- Initial baseline commit: `595da2d`
- Local generated state: `.studio/` (ignored)
- External `animation_editor_live`: read-only reference; never edit it
- External `Semantic tile generator design`: read-only reference; never edit it

This is a clean-room project. Do not add either external repository as a
runtime dependency or write generated assets back into them.

## Implemented now

| Area | Current state |
| --- | --- |
| Desktop shell | Tauri 2 shell with a Svelte 5 UI |
| Workflow UI | Six-stage navigation and polished three-panel layout |
| Brief | Editable name, mob/NPC kind, and description |
| Prompt | Live contract-constrained prompt preview |
| UI session | Creates an atomic durable session and advances to Concept |
| Session restore | Reopens the newest valid local session on desktop startup |
| Saved identity | Shows session id, revision, contract id, and saved workspace |
| Candidate intake | Imports a local 32 x 32 transparent PNG as a new immutable Concept revision |
| Candidate restore | Reopens, lists, and reads saved candidates through either adapter |
| Candidate comparison | Switches revisions and previews them at 1x, 8x, and 16x |
| Candidate evidence | Shows provenance, SHA-256 identity, and structural-intake status without implying visual acceptance |
| Contract validation | Measures six PNG rules locally and reports ground luma as Not assessed pending a pinned reference |
| Validation UI | Shows Pass, Fail, and Not assessed totals plus per-rule evidence for the selected revision |
| Concept selection | Turnaround records the exact selected Concept id, SHA-256, user authority, and selection time |
| Turnaround storage | Atomically preserves canonical down/right/up/left PNGs as one immutable revision |
| Turnaround restore | Desktop restart reopens the latest four-view comparison and its selected Concept receipt |
| Turnaround evidence | Rehashes all four sources and recomputes per-direction plus aggregate structural reports |
| Contract | JSON Schema, versioned JSON instance, and TypeScript representation |
| Approval | Human-only approval boundary shown in UI, contract, prompt, and MCP |
| MCP | Stdio and localhost Streamable HTTP transports |
| MCP tools | Thirteen tools covering contract, prompt, sessions, Concept intake/read/validation, and Turnaround create/read/validation |
| Shared storage | Tauri and MCP adapters use one atomic `.studio/sessions` protocol for immutable Concepts and Turnarounds |
| Compatibility | Shared session, Concept, Turnaround, and validation fixtures plus TypeScript and Rust failure-path tests |
| Agent guidance | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, project rules, and skill |
| App icon | Generated Tauri desktop/mobile icon set from `src-tauri/icons/icon.svg` |

## Not implemented yet

- No image-generation provider adapter is integrated.
- Ground-luma validation remains Not assessed until a pinned ground reference
  exists.
- No pinned TileForge reference pack exists yet.
- Walk Cycle, world-test, and export stages are visual shells.
- No approved-candidate promotion or publishing operation exists.
- Tauri bundling/installers are disabled in `tauri.conf.json`.
- The contract JSON import is typed, but full JSON-Schema validation is not yet
  an automated test.

Do not describe any item in this section as working until it has implementation
and verification evidence.

## Architecture snapshot

- `src/` contains the Svelte UI and client-neutral TypeScript domain helpers.
- `src-tauri/` contains the native shell and durable session commands.
- `contracts/` is the versioned machine-readable art/world contract.
- `mcp/` exposes the domain over MCP and uses the shared filesystem protocol.
- `.studio/` is the ignored local workspace for sessions, immutable Concepts,
  Turnarounds, and generation evidence.

The shared boundary is documented in `docs/DECISIONS.md`: both thin adapters
use the same session, Concept, and Turnaround documents, identity rules, brief
limits, directory layout, hash checks, atomic publish behavior, and recomputed
artifact-hash-bound validation reports.
`tests/fixtures/session-v1.json` and
`tests/fixtures/concept-candidate-v1.json` guard storage compatibility;
`tests/fixtures/turnaround-candidate-v1.json` guards the user-selection and
four-direction artifact contract; `tests/fixtures/validation-report-v1.json`
guards validator compatibility.

## Current local review gate

Ignored `.studio/` state currently contains the first real M04 Turnaround and
its first immutable direction repair:

- session: `mirelight-pilgrim-20260726212353-df9dd645`
- user-selected Concept:
  `concept-r0004-20260726221830-633afb02`
- preserved original Turnaround:
  `turnaround-r0001-20260726224205-a558350a`
- current review Turnaround:
  `turnaround-r0002-20260726225909-696334cf`
- repair scope: the user rejected the r1 right-facing outline/edge bleed; r2
  replaces only `right.png` with a deterministic mirrored and re-anchored copy
  of the user-accepted-good left profile
- byte preservation: r2 down, up, and left SHA-256 identities exactly match r1;
  r1 remains unchanged
- structural evidence: 24 Pass / 0 Fail / 4 Not assessed, with one ground-luma
  result pending per direction
- identity consistency: Not assessed, user authority

The built-in ImageGen sources and deterministic repair evidence are preserved
under `.studio/generated-source/`. The r2 repair used no AI service, API key, or
incremental billing. Do not begin Walk Cycle work until the user explicitly
accepts r2 identity consistency or asks for another immutable Turnaround
revision.

## Verification evidence

The M04 Turnaround slice was verified on Windows with Node 24.15.0, npm
11.12.1, and Rust 1.95:

- `npm run check` — 0 errors and 0 warnings
- `npm run build` — production bundle built
- `npm run test:mcp` — thirteen tools, locked approval contract, shared
  session/Concept/Turnaround/report compatibility, exact selected-down
  preservation, immutable four-view reads, independent rule failures,
  collisions, and atomic failure cleanup passed
- `npm run test:mcp:stdio` — real stdio transport passed
- `npm run test:mcp:http` — real localhost HTTP transport passed while server ran
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — passed
- `cargo check --manifest-path src-tauri/Cargo.toml` — passed
- `cargo test --manifest-path src-tauri/Cargo.toml` — twelve
  session/Concept/Turnaround/validation compatibility and failure-path tests
  passed
- `npm audit --audit-level=moderate` — 0 vulnerabilities
- Native desktop QA — after a full app restart, the app restored the exact
  Concept r4 selection receipt and Turnaround r1, displayed all four views,
  reported 24 Pass / 0 Fail / 4 Not assessed, and kept identity consistency
  Not assessed and user-only
- Local r2 repair evidence — the rejected r1 right view was replaced in a new
  atomic Turnaround; down, up, and left hashes remain exact; validation reports
  6 Pass / 0 Fail / 1 Not assessed per direction and 24 / 0 / 4 in aggregate

Re-run checks relevant to any new change. For the HTTP smoke test, start
`npm run mcp:http` in a separate terminal first.

## Recommended next milestone

The next action is the user-owned Turnaround r2 identity gate. If the user
accepts the current down/right/up/left set, continue M04 with an immutable
four-frame walk cycle per direction at 300 ms. If the user requests changes,
create another Turnaround revision; never overwrite r1 or r2. Do not treat 24
green structural checks as identity acceptance.

Any future AI integration must be included in the user's existing
subscriptions. Do not enable pay-as-you-go APIs, purchased credits,
usage-metered billing, or paid add-ons.

## Handoff discipline

When completing a milestone:

1. update this file’s implemented/not-implemented sections;
2. update `docs/ROADMAP.md`;
3. record architecture decisions in `docs/DECISIONS.md`;
4. run and report the relevant checks;
5. preserve unrelated user changes;
6. do not commit or push unless the user explicitly requests it.
