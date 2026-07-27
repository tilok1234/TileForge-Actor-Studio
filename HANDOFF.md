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
- Windows local generated state:
  `%LOCALAPPDATA%\TileForge\Actor Studio\.studio`
- Source-checkout `.studio/`: ignored migration backup, not the packaged
  default
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
| Contract validation | Measures six PNG rules locally; earlier stages keep ground luma Not assessed until World Test supplies a pinned placement |
| Validation UI | Shows Pass, Fail, and Not assessed totals plus per-rule evidence for the selected revision |
| Concept selection | Turnaround records the exact selected Concept id, SHA-256, user authority, and selection time |
| Turnaround storage | Atomically preserves canonical down/right/up/left PNGs as one immutable revision |
| Turnaround restore | Desktop restart reopens the latest four-view comparison and its selected Concept receipt |
| Turnaround evidence | Rehashes all four sources and recomputes per-direction plus aggregate structural reports |
| Turnaround acceptance | Walk Cycle records the exact accepted Turnaround id, four source hashes, user authority, and acceptance time |
| Walk Cycle storage | Atomically preserves four frames per down/right/up/left direction as one immutable revision at 300 ms |
| Walk Cycle restore | Desktop restart opens Animate, plays all four directions, and shows the accepted Turnaround receipt |
| Walk Cycle evidence | Rehashes all sixteen frame sources and recomputes per-frame plus aggregate structural reports |
| Walk Cycle acceptance | World Test records the exact accepted Walk Cycle id, all sixteen source hashes, user authority, and acceptance time |
| Reference pack | Tracks four TileForge scenes across four themes at 1x with upstream commits, dimensions, byte lengths, and SHA-256 identities |
| World Test storage | Locally composites and atomically preserves sixteen immutable 640 x 384 scene/theme previews |
| World Test restore | Desktop restart opens World Test r1, restores its accepted Walk Cycle receipt, and switches all pinned scene/theme evidence |
| World Test evidence | Rehashes all previews and recomputes 256 frame-to-ground luma measurements with final-art judgment user-only |
| Final-art receipt | Export records the exact approved World Test document hash, sixteen preview identities, user authority, and approval time |
| Export storage | Atomically preserves one immutable draft with `export.json`, `sprite-sheet.png`, `metadata.json`, and `provenance.json` |
| Export restore | Desktop restart opens Export r1, shows its full stable id and 4 x 4 sheet, and keeps publishing visibly unapproved |
| Export evidence | Rehashes all package files, reconstructs sheet pixels from sixteen sources, and verifies metadata, provenance, and publishing boundary |
| Export access | Desktop validates the exact immutable package before opening its directory in Windows Explorer |
| Packaged storage | Desktop and MCP default to the same uninstall-safe `%LOCALAPPDATA%\TileForge\Actor Studio\.studio`; `TFAS_WORKSPACE` retains precedence |
| Windows release | Current-user NSIS installer builds without administrator rights |
| Contract | JSON Schema, versioned JSON instance, and TypeScript representation |
| Approval | Human-only approval boundary shown in UI, contract, prompt, and MCP |
| MCP | Stdio and localhost Streamable HTTP transports |
| MCP tools | Twenty-five tools covering contract, prompt, sessions, Concept, Turnaround, Walk Cycle, World Test, and Export create/read/validation |
| Shared storage | Tauri and MCP adapters use one atomic `.studio/sessions` protocol for immutable Concepts, Turnarounds, Walk Cycles, World Tests, and Exports |
| Compatibility | Shared session, Concept, Turnaround, Walk Cycle, World Test, Export, and validation fixtures plus TypeScript and Rust failure-path tests |
| Agent guidance | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, project rules, and skill |
| App icon | Generated Tauri desktop/mobile icon set from `src-tauri/icons/icon.svg` |

## Not implemented yet

- No image-generation provider adapter is integrated.
- Earlier-stage ground-luma validation remains Not assessed because those
  artifacts have no pinned placement; World Test resolves it with real grounds.
- No publishing operation exists; every prepared Export remains a local draft.
- The contract JSON import is typed, but full JSON-Schema validation is not yet
  an automated test.

Do not describe any item in this section as working until it has implementation
and verification evidence.

## Architecture snapshot

- `src/` contains the Svelte UI and client-neutral TypeScript domain helpers.
- `src-tauri/` contains the native shell and durable session commands.
- `contracts/` is the versioned machine-readable art/world contract.
- `mcp/` exposes the domain over MCP and uses the shared filesystem protocol.
- `reference-packs/` contains the copied, tracked, SHA-256-pinned TileForge
  World Test input.
- `%LOCALAPPDATA%\TileForge\Actor Studio\.studio` is the packaged Windows
  workspace for sessions, immutable Concepts, Turnarounds, Walk Cycles, World
  Tests, Exports, and generation evidence. `TFAS_WORKSPACE` redirects both
  adapters. The source-checkout `.studio/` remains an ignored migration backup.

The shared boundary is documented in `docs/DECISIONS.md`: both thin adapters
use the same session, Concept, Turnaround, Walk Cycle, World Test, and Export
documents, identity rules, brief limits, directory layout, hash checks, atomic
publish behavior, and recomputed artifact-hash-bound validation reports.
`tests/fixtures/session-v1.json` and
`tests/fixtures/concept-candidate-v1.json` guard storage compatibility;
`tests/fixtures/turnaround-candidate-v1.json` guards the user-selection and
four-direction artifact contract;
`tests/fixtures/walk-cycle-candidate-v1.json` guards the accepted-Turnaround
receipt, timing, and sixteen-frame contract;
`tests/fixtures/world-test-candidate-v1.json` guards the accepted-Walk-Cycle
receipt, pinned pack, sixteen-preview, no-cost, and final-art boundary;
`tests/fixtures/export-candidate-v1.json` guards the user-approved World Test
receipt, 4 x 4 package layout, draft status, no-cost preparation, and
publishing boundary;
`tests/fixtures/validation-report-v1.json` guards validator compatibility.

## Current local artifact and publishing gate

The post-release generality run is active with a deliberately different mob:

- session: `orc-vanguard-20260727012850-6bb50608`
- brief: Orc Vanguard, a broad green-skinned front-liner with a horned helmet,
  asymmetrical armor, and crimson cloth, shown unarmed for the v1 walk scope
- stage: Animate motion/readability gate
- immutable Concept candidates:
  - A: `concept-r0001-20260727013308-fd1e91fe`
  - B: `concept-r0002-20260727013308-e6d9ca44`
  - C: `concept-r0003-20260727013308-18fe2f93`
- structural evidence: each candidate reports 6 Pass / 0 Fail / 1 Not
  assessed; ground luma remains pending until World Test
- user-selected Concept: A,
  `concept-r0001-20260727013308-fd1e91fe`
- immutable Turnaround:
  `turnaround-r0001-20260727014752-775090c7`
- source receipt: exact Concept A id and SHA-256
  `95d3b5a484e700f8e266651ac55f85d23e2c305f6df7101f9fcb440509baf568`,
  `selectedBy: user`
- byte preservation: `down.png` exactly matches the selected Concept A hash;
  right, up, and left are separate immutable sources
- Turnaround structural evidence: 24 Pass / 0 Fail / 4 Not assessed, with one
  ground-luma result pending per direction
- identity consistency: explicitly accepted by the user in chat with "looks
  good"; this unlocks animation only and is not final-art approval
- preserved rejected Walk Cycle:
  `walk-cycle-r0001-20260727015657-df64eb2e`
- source receipt: exact Turnaround r1 id and all four direction hashes,
  `acceptedBy: user`
- clip: `walk`, four frames per down/right/up/left direction, 300 ms
- r1 rejection: the user explicitly rejected motion/readability because the
  actor only wiggled back and forth and its feet did not walk; the user said
  the remaining motion was pretty good
- preserved motion-improved Walk Cycle:
  `walk-cycle-r0002-20260727020559-adbbcf52`
- r2 motion: deterministic neutral/first-foot/neutral/opposite-foot rhythm;
  down and up visibly lift and offset alternating boots, right and left move
  the lower foot through a two-pixel forward/back arc, and the restrained
  armored torso/arm sway from r1 remains
- r2 feedback: the user confirmed the walking motion was better, then requested
  cleanup of the thin planted-heel spikes and isolated side pixels
- preserved silhouette-cleaned Walk Cycle:
  `walk-cycle-r0003-20260727021420-5e86e86e`
- r3 cleanup: preserves r2 foot motion, removes artificial arm-tip pixels and
  redundant torso/hip seam nubs, and uses a connected three-pixel planted boot
  core; every final frame is one four-connected visible component
- r3 feedback: the user requested more vertical bob while walking
- preserved heavier-bobbing Walk Cycle:
  `walk-cycle-r0004-20260727021942-2ebffe25`
- r4 bob: preserves r3's cleaned silhouettes and foot motion, adds a one-pixel
  downward upper-body weight drop on both passing-foot poses, and returns to
  the exact neutral on the alternating frames; every final frame remains one
  four-connected visible component
- r4 feedback: the user reported that the foot broke in some frames
- active immutable Walk Cycle:
  `walk-cycle-r0005-20260727043425-305dceaf`
- r5 foot repair: preserves r4's weight drop, keeps each ankle/lower-leg column
  fixed and connected, and restricts motion to the bottom three foot rows;
  every final frame remains one four-connected visible component
- frames 0 and 2 are intentional neutral beats, and frame 0 in every direction
  is byte-identical to Turnaround r1
- Walk Cycle structural evidence: 96 Pass / 0 Fail / 16 Not assessed, with one
  ground-luma result pending per frame
- motion and readability: Not assessed with user authority; the next action is
  the user's acceptance or rejection of exact Walk Cycle r5,
  and no World Test may be created before that decision
- generation boundary: sources used OpenAI built-in ImageGen through the
  user's subscription; preparation and validation were local, with no API key,
  paid add-on, or usage-metered service
- preservation: the fourteen Concept evidence files remain under
  `generated-source/orc-vanguard-20260727`; sixteen Turnaround identity
  references, prompts, sources, prepared views, comparison, and import files
  were copied byte-for-byte under
  `generated-source/orc-vanguard-turnaround-20260727`; sixty-one r1 Walk Cycle
  source, preparation, rejected-working-preview, final-frame, review, and
  import files were copied byte-for-byte under
  `generated-source/orc-vanguard-walk-cycle-20260727`; sixty-one r2 source,
  planted-anchor repair, preserved working preview, final frame, review, and
  import files were copied byte-for-byte under
  `generated-source/orc-vanguard-walk-cycle-r2-20260727`; sixty-one r3 source,
  silhouette-cleanup, preserved working preview, final frame, review, and
  import files were copied byte-for-byte under
  `generated-source/orc-vanguard-walk-cycle-r3-20260727`; twenty-five r4
  source, bob-preparation, final-frame, review, and import files were copied
  byte-for-byte under
  `generated-source/orc-vanguard-walk-cycle-r4-20260727`; twenty-five r5
  source, stable-ankle preparation, final-frame, review, and import files were
  copied byte-for-byte under
  `generated-source/orc-vanguard-walk-cycle-r5-20260727`
- animation preparation: local deterministic pixel motion,
  `additionalAiCost: false`; no AI service was used
- scope translation: the user's larger eight-frame axe-swing reference informed
  character language only; v1 keeps the actor at 32 px, empty-handed, and
  limited to the four-direction walk workflow

The earlier Snowberry Courier session
`snowberry-courier-20260727010001-2a512f14` and its three unselected immutable
Concepts remain preserved and unchanged. It is paused rather than deleted or
promoted so that only the Orc Vanguard is active.

Ignored `.studio/` state currently contains the first real M04 Turnaround and
its first immutable direction repair:

- session: `mirelight-pilgrim-20260726212353-df9dd645`
- user-selected Concept:
  `concept-r0004-20260726221830-633afb02`
- preserved original Turnaround:
  `turnaround-r0001-20260726224205-a558350a`
- accepted Turnaround:
  `turnaround-r0002-20260726225909-696334cf`
- repair scope: the user rejected the r1 right-facing outline/edge bleed; r2
  replaces only `right.png` with a deterministic mirrored and re-anchored copy
  of the user-accepted-good left profile
- byte preservation: r2 down, up, and left SHA-256 identities exactly match r1;
  r1 remains unchanged
- structural evidence: 24 Pass / 0 Fail / 4 Not assessed, with one ground-luma
  result pending per direction
- identity consistency: explicitly accepted by the user in chat for animation

The first immutable Animate-stage candidate is now:

- Walk Cycle:
  `walk-cycle-r0001-20260726232040-8f002087`
- source receipt: exact Turnaround r2 id and down/right/up/left hashes,
  `acceptedBy: user`
- clip: `walk`, four frames per direction, 300 ms
- structural evidence: 96 Pass / 0 Fail / 16 Not assessed, with one
  ground-luma result pending per frame
- motion and readability: explicitly accepted by the user in chat for World
  Test; this is not final-art approval

The first immutable World Test candidate is now:

- World Test:
  `world-test-r0001-20260726235243-bb4e1c16`
- source receipt: exact Walk Cycle r1 id plus all sixteen source hashes,
  `acceptedBy: user`
- reference receipt: `tileforge-world-test-v1`, manifest SHA-256
  `91d2a7f50ba9626c5ba5b9b78802a750682f7958b52b65316147012caf87a535`,
  source checkout `3eb01d0b5cc3a59a0327a26e3f8c416401fc3c4c`, generated
  engine commit `199ed7d`
- previews: Scale Lineup, Forest Clearing, Crownhold, and Tidewater across
  forest, autumn, dusk, and winter; sixteen immutable 640 x 384 PNGs
- ground evidence: 240 Pass / 16 Fail / 0 Not assessed across 256 ordered
  frame/reference measurements
- failure concentration: eight frames fail the mean-luma proxy on dusk Scale
  Lineup and the same eight fail on dusk Forest Clearing; all other fourteen
  references pass all sixteen frames
- final art: explicitly approved by the user in chat for the exact first
  Export; the immutable World Test remains unchanged and carries no mutable
  approval field

The first immutable draft Export is now:

- Export: `export-r0001-20260727001749-a13580f4`
- final-art receipt: exact World Test r1 id, `world-test.json` SHA-256
  `6410e02e0df0e27d114d9ff4cf354bbc072962e1478edd36421504d5c5975a30`,
  all sixteen preview identities, `approvedBy: user`, and approval time
- source receipt: exact Walk Cycle r1 id and all sixteen source frame hashes
- package: `export.json`, 128 x 128 `sprite-sheet.png`, `metadata.json`, and
  `provenance.json`
- sheet layout: down/right/up/left rows, frame 0–3 columns, 32 x 32 cells
- sheet SHA-256:
  `f2a6734a63b7d762b258f9dec9d56b0e0a152ed7f843eee40dff516d60a2ac4e`
- package evidence: 7 Pass / 0 Fail / 0 Not assessed; every sheet cell is
  pixel-identical to its immutable source
- preparation: local deterministic sheet builder,
  `additionalAiCost: false`
- status: `draft`; publishing is `not_approved`, user authority, and no
  publishing operation exists

The built-in ImageGen sources, deterministic Turnaround repair, and
deterministic cloak-sway Walk Cycle and World Test review evidence are
preserved in the shared local workspace. Neither r2, Walk Cycle r1, World Test
r1, Export r1, nor release hardening used an AI API, additional AI service, or
incremental billing.

## Verification evidence

The M04 through M06 slices were verified on Windows with Node 24.15.0,
npm 11.12.1, and Rust 1.95:

- `npm run check` — 0 errors and 0 warnings
- `npm run build` — production bundle built
- `npm run test:mcp` — twenty-five tools, locked approval contract, shared
  session/Concept/Turnaround/Walk Cycle/World Test/Export/report compatibility,
  exact transition-source preservation, immutable artifact reads, 256 ground
  measurements, Export sheet reconstruction, JSON receipt checks, independent
  failures, collisions, tamper detection, and atomic failure cleanup passed
- `npm run test:mcp:stdio` — real stdio transport passed
- `npm run test:mcp:http` — real localhost HTTP transport passed while server ran
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — passed
- `cargo check --manifest-path src-tauri/Cargo.toml` — passed
- `cargo test --manifest-path src-tauri/Cargo.toml` — twenty
  session/Concept/Turnaround/Walk Cycle/World Test/Export/validation
  compatibility, workspace resolution, safe folder-opening, and failure-path
  tests passed
- `npm audit --audit-level=moderate` — 0 vulnerabilities
- Native desktop QA — after a full app restart, the app restored the exact
  Concept r4 selection receipt and Turnaround r1, displayed all four views,
  reported 24 Pass / 0 Fail / 4 Not assessed, and kept identity consistency
  Not assessed and user-only
- Local r2 repair evidence — the rejected r1 right view was replaced in a new
  atomic Turnaround; down, up, and left hashes remain exact; validation reports
  6 Pass / 0 Fail / 1 Not assessed per direction and 24 / 0 / 4 in aggregate
- Native Walk Cycle QA — after a full app restart, the desktop restored Animate
  and Walk Cycle r1, displayed the exact Turnaround r2 acceptance receipt,
  advanced all four direction previews at 300 ms, reported 96 Pass / 0 Fail /
  16 Not assessed, and kept motion/readability Not assessed and user-only
- Native World Test QA — after a full app restart, the desktop restored World
  Test r1, displayed the exact accepted Walk Cycle receipt and real Scale
  Lineup preview, reported 240 Pass / 16 Fail / 0 Not assessed, and kept final
  art Not assessed and user-only
- Native Export QA — after a full app restart, the desktop restored Export r1,
  displayed the 4 x 4 sheet, user final-art receipt, 7 Pass / 0 Fail / 0 Not
  assessed package evidence, and `not_approved` publishing boundary; a
  subsequent native observation confirmed the full stable Export id is visibly
  rendered
- Release build — `npm run tauri -- build --bundles nsis` produced the standalone
  release executable and
  `TileForge Actor Studio_0.1.0_x64-setup.exe`
- Final installer SHA-256:
  `0c52a295c8966837a9a69cec518704c0a151e80a7cace087b071c708bf4c2dfb`
- Fresh-install QA — the current-user NSIS installer exited successfully,
  registered version 0.1.0, and the installed executable launched from
  `%LOCALAPPDATA%\TileForge Actor Studio`
- Installed-state QA — the packaged app restored Mirelight Pilgrim Export r1
  from `%LOCALAPPDATA%\TileForge\Actor Studio\.studio`, displayed its full id,
  7 Pass / 0 Fail / 0 Not assessed evidence, closed publishing gate, and Open
  Folder control

Re-run checks relevant to any new change. For the HTTP smoke test, start
`npm run mcp:http` in a separate terminal first.

## Recommended next milestone

M06 release hardening is complete. The second-actor generality test is now at
the Orc Vanguard Animate motion/readability gate. The next action is the user's
acceptance or rejection of
`walk-cycle-r0005-20260727043425-305dceaf`. If accepted, create a World Test
tied to that exact sixteen-frame receipt and the pinned reference pack. If
rejected, create a new immutable Walk Cycle revision and preserve the current
candidate. Keep this one mob active and preserve every revision. Publishing
remains a separate user-owned scope decision and is not implemented in version
1; do not add a destination or publish without explicit authority.

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
