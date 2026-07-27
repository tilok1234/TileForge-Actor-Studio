# TileForge Actor Studio Handoff

This is the canonical continuation point for a new agent or chat. It is an
operational snapshot, not a substitute for the live Git state.

## Start here

Work only in:

```text
C:\Users\headc\Documents\TileForge-Actor-Studio
```

Before editing, run:

```powershell
Get-Location
git status -sb
git log -5 --oneline --decorate
git remote -v
```

Then read completely, in order:

1. `AGENTS.md`
2. this handoff
3. `README.md`
4. `contracts/tileforge-actor-32-v1.json`
5. `docs/ARCHITECTURE.md`
6. `docs/AGENT_WORKFLOW.md`
7. `docs/ROADMAP.md`
8. `docs/DECISIONS.md`
9. `docs/MCP.md`
10. `docs/START_NEW_CHAT.md`

Reconcile every claim below with the live branch, working tree, package
versions, and tests. Preserve dirty work and report any disagreement before
editing.

## Verified source checkpoint

As of 2026-07-28:

- branch: `main`
- M07 implementation commit: `81f443e`
  (`Add subscription-native generation requests`)
- documentation audit commit: `0a54b33`
  (`Refresh handoff for installed M07 proof`)
- the containing checkpoint completes the M08 installed proof and application
  version `0.1.1`
- expected remote: `https://github.com/tilok1234/TileForge-Actor-Studio.git`
- expected state after the M08 checkpoint: clean and synchronized with
  `origin/main`
- source and installed milestone: M08 complete; the post-proof actor is at the
  Walk Cycle motion/readability gate
- MCP surface: 28 tools

These are orientation expectations, not permission to skip live verification.

## Product and authority

TileForge Actor Studio is deliberately limited to one 32 px mob or NPC at a
time:

```text
Brief -> Concept -> Turnaround -> Animate -> World test -> Export
```

The contract supplies deterministic world boundaries. Agents may compile
prompts, create and compare immutable candidates, run structural checks, and
prepare local drafts. Only the user may:

- select a Concept for Turnaround;
- accept Turnaround identity consistency;
- accept Walk Cycle motion and readability;
- approve final art after World Test;
- approve publishing.

No publishing operation exists. A prepared Export is still a local draft.

## Hard boundaries

- Never modify `C:\Users\headc\Documents\animation_editor_live`.
- Never modify
  `C:\Users\headc\Documents\Semantic tile generator design`.
- Treat both external repositories as read-only evidence.
- Never overwrite a generated candidate or repair an old revision in place.
- Do not expand version 1 into maps, tiles, bosses, effects, attacks,
  equipment, paperdolls, batching, or publishing.
- Do not add secrets or provider credentials.
- Use only AI capabilities included in the user's existing subscriptions.
  Pay-as-you-go APIs, purchased credits, usage billing, and paid add-ons are
  forbidden.

## Shared persistence boundary

Desktop and MCP are thin adapters over one local filesystem protocol. On
packaged Windows builds the default root is:

```text
%LOCALAPPDATA%\TileForge\Actor Studio\.studio
```

`TFAS_WORKSPACE` has highest precedence for both adapters. Non-Windows source
development falls back to the ignored repository `.studio/`.

Each session has immutable subdirectories for:

- Concept generation requests;
- Concept candidates;
- Turnarounds;
- Walk Cycles;
- World Tests;
- draft Exports.

Complete records are written to hidden same-parent temporary directories and
published with one rename. Readers rehash stored artifacts. Identity
collisions fail instead of overwriting.

## Implemented through M08

### Brief and durable session

- The Svelte/Tauri desktop creates an atomic versioned session and advances to
  Concept.
- The UI shows the saved session id, revision, contract id, and workspace.
- Restart restores the newest valid local session.
- MCP can create, list, and read the same sessions.

### Subscription-native generation handoff

- Starting a desktop Concept automatically creates the first immutable
  `generation-requests/<request-id>/request.json`.
- The request records the exact compiled prompt, one through four separate
  32 x 32 down-facing outputs, the immutable Concept import continuation, the
  no-additional-cost rule, and user-only approval.
- The desktop defaults to three outputs, displays the stable request id, and
  may prepare another immutable request revision.
- Restart restores the newest request; MCP can create, list, and read the same
  document.
- The request is a durable handoff, not a dispatcher or background job.
  Creating it does not invoke Codex, Claude, Antigravity, or an image provider.
- A connected client may fulfill it only when that client actually has an
  included native image capability. Each PNG is imported separately through
  `import_concept_candidate`.
- Actor Studio has no provider API, API key, fulfillment worker, mutable
  request status, or paid fallback.

### Installed cross-client generation proof

- All application version fields are `0.1.1`, including only the root
  JavaScript lock entries and the Actor Studio Rust package entry.
- A new current-user NSIS package was built from M07 source, installed over the
  historical `0.1.0` package, and verified from its installed executable.
- The installed desktop created a fresh Concept session and generation request,
  displayed both stable identities, and restored the exact request after a
  full close and restart.
- A separately connected MCP HTTP client listed that request as newest and
  read the exact immutable prompt and three-output contract from the shared
  per-user workspace.
- Codex's built-in subscription ImageGen was confirmed live and invoked three
  separate times with the exact stored prompt. The large returned previews
  were converted locally into three hard-alpha, contract-sized PNGs without an
  additional AI service.
- MCP imported all three results with generated provenance as immutable
  unreviewed Concept revisions. Each reports 6 Pass / 0 Fail / 1 Not assessed;
  ground contrast waits for World Test and visual judgment remains user-only.
- After another installed restart, the desktop restored all three revisions,
  displayed each at 8x, and kept the workflow at Concept.

### Concept through Export

- Concept intake preserves each original 32 x 32 transparent PNG as an
  unreviewed immutable revision with provenance and SHA-256 identity.
- Structural validation measures canvas, alpha, visible height, placement
  contact, palette, clipping, and later ground luma without implying visual
  approval.
- Turnaround records the user's selected Concept receipt and atomically
  preserves down/right/up/left views.
- Walk Cycle records the user's accepted Turnaround receipt and preserves four
  frames per direction at 300 ms. Frame 0 remains byte-identical to its
  accepted Turnaround view.
- Static Concept/Turnaround art must contact `(16, 28)`. Animated frames may
  contact anywhere on row 28 so either foot can lift.
- World Test binds an accepted Walk Cycle to the tracked SHA-256-pinned
  TileForge reference pack, creates sixteen local scene/theme previews, and
  computes 256 frame-to-ground measurements.
- Export records the user's exact World Test approval receipt and prepares an
  immutable 128 x 128 sheet, metadata, provenance, and export document.
- Export validation reconstructs the sheet from all sixteen source frames.
- Every Export remains `draft`; publishing stays `not_approved`.

## Durable actor evidence

The local workspace contains multiple preserved sessions. Do not delete,
promote, or rewrite them during the next milestone.

### Mosscap Scout - installed cross-client proof

- session: `mosscap-scout-20260727225718-dc51655e`
- generation request:
  `concept-gen-r0001-20260727225718-749783a6`
- requested outputs: three separate 32 x 32 down-facing candidates
- immutable Concept r1:
  `concept-r0001-20260727230409-aa483745`
  (`6abe270ea4c7c12668cbf9d0205fddd618c5117e5e5510bce34013077edbd116`)
- immutable Concept r2:
  `concept-r0002-20260727230409-e9e46a75`
  (`46b0dbd06cc32e1d3013302f54daca6d87477290108d301e7dfcbd3983fdc8af`)
- immutable Concept r3:
  `concept-r0003-20260727230409-b9b4f0de`
  (`a94de8554d5a8c2c2ea199b193f9b73d5cac9f3b2ada3dda49d2adc41f0ec10e`)
- every candidate is 32 x 32, 27 visible pixels tall, hard-alpha, uses 16
  visible colors, contacts `(16, 28)`, avoids frame-edge clipping, and remains
  `unreviewed`
- every structural report is 6 Pass / 0 Fail / 1 Not assessed; visual judgment
  is `not_assessed` with user authority
- the user explicitly selected Concept r1:
  `concept-r0001-20260727230409-aa483745`
- immutable Turnaround r1:
  `turnaround-r0001-20260727231939-6d597fb3`
- Turnaround r1 preserves the selected down PNG byte for byte and stores new
  right, up, and left views with generated provenance
- Turnaround validation reports 24 Pass / 0 Fail / 4 Not assessed
- the user explicitly accepted exact Turnaround r1 for animation
- immutable Walk Cycle r1:
  `walk-cycle-r0001-20260727232551-4db453a0`
- Walk Cycle r1 records exact Turnaround r1 with `acceptedBy: user`, preserves
  every direction byte for byte as frame 0, and uses a 300 ms
  neutral/step/neutral/opposite-step loop
- the user rejected Walk Cycle r1 because only the feet moved; r1 remains
  immutable diagnostic evidence
- immutable replacement Walk Cycle r2:
  `walk-cycle-r0002-20260727233042-abdbb047`
- r2 preserves r1's foot motion and all accepted Turnaround sources as frames
  0 and 2, while moving the upper body down one pixel on frames 1 and 3 to add
  a restrained walking bob
- all sixteen r2 frames remain 32 x 32, hard-alpha, at or below 16 colors,
  grounded on row 28, unclipped, and one connected silhouette
- r2 validation reports 96 Pass / 0 Fail / 16 Not assessed; motion and
  readability are `not_assessed` with user authority
- no World Test is authorized until the user explicitly accepts this exact
  Walk Cycle r2

### Orc Vanguard - completed generality proof

- session: `orc-vanguard-20260727012850-6bb50608`
- selected Concept:
  `concept-r0001-20260727013308-fd1e91fe`
- accepted Turnaround:
  `turnaround-r0001-20260727014752-775090c7`
- accepted Walk Cycle:
  `walk-cycle-r0011-20260727064901-0ebf795f`
- approved World Test:
  `world-test-r0001-20260727065711-23d42e04`
- draft Export:
  `export-r0001-20260727070603-f9fe69a3`
- Export sheet SHA-256:
  `c2884921f552992ce1339ff7b27b2ff8ce9e4e06a9f074d351ac8f9759e1c057`
- final evidence: World Test 256 Pass / 0 Fail / 0 Not assessed; Export
  7 Pass / 0 Fail / 0 Not assessed
- publishing: `not_approved`

Walk Cycle r1 through r10 remain preserved as rejected or superseded
diagnostic revisions. R11 is the user-accepted motion/readability revision.
The user approved exact World Test r1 as final art for the draft Export, not
for publishing.

### Mirelight Pilgrim - first complete workflow

- session: `mirelight-pilgrim-20260726212353-df9dd645`
- selected Concept:
  `concept-r0004-20260726221830-633afb02`
- accepted Turnaround:
  `turnaround-r0002-20260726225909-696334cf`
- accepted Walk Cycle:
  `walk-cycle-r0001-20260726232040-8f002087`
- approved World Test:
  `world-test-r0001-20260726235243-bb4e1c16`
- draft Export:
  `export-r0001-20260727001749-a13580f4`
- World Test evidence: 240 Pass / 16 Fail / 0 Not assessed, with all
  failures on dusk grass in Scale Lineup and Forest Clearing
- Export evidence: 7 Pass / 0 Fail / 0 Not assessed
- publishing: `not_approved`

Turnaround r1 remains preserved because its right-facing outline was rejected.
R2 changed only the right view and preserved the other three sources byte for
byte.

### Other preserved work

- Snowberry Courier is paused with three unselected immutable Concepts.
- The isolated M07 native QA session under `C:\tmp\tfas-m07-ui-qa` was
  intentionally temporary and removed after verification.
- Ignored `generated-source/` evidence preserves prompts, sources, comparisons,
  and review artifacts from the completed actor work.
- `C:\tmp\tfas-m08-mosscap` preserves the three locally prepared M08 inputs and
  their comparison sheet for the current visual-selection gate. The durable
  candidate copies live in the shared per-user workspace.
- ignored repository-local
  `.studio\mosscap-r1-turnaround\turnaround-comparison.png` preserves the
  down/right/up/left review sheet and headless preparation evidence for the
  accepted Turnaround
- ignored repository-local `.studio\mosscap-r1-walk` preserves all sixteen
  prepared frame inputs, structural preparation evidence, a static review
  sheet, and animated GIFs for the rejected feet-only candidate
- ignored repository-local `.studio\mosscap-r2-walk` preserves all sixteen
  bobbing repair inputs, structural preparation evidence, the MCP publication
  receipt, a static review sheet, and the animated GIF for the current
  motion/readability gate

## Installed release proof

- Source `main` retains M07 implementation commit `81f443e` and adds the M08
  release/version checkpoint.
- `package.json`, the two root `package-lock.json` entries,
  `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and the Actor Studio
  package entry in `src-tauri/Cargo.lock` are consistently `0.1.1`.
- Current installer:
  `src-tauri\target\release\bundle\nsis\TileForge Actor Studio_0.1.1_x64-setup.exe`
- installer bytes: `3295196`
- installer SHA-256:
  `b9686db13d425083a9c8d55cf558c2796dd18d7c40c6ccd19bfea1292a97ff96`
- installed executable:
  `%LOCALAPPDATA%\TileForge Actor Studio\tileforge-actor-studio.exe`
- installed executable product/file version: `0.1.1`
- installed executable bytes: `11508736`
- installed executable SHA-256:
  `78e965b8c6adc392f385967043ca28156af1390097760c8cc6c59d92f374be5f`
- the current-user installer completed with exit code 0, restored the existing
  uninstall-safe workspace, and contains the M07 request UI and behavior.
- The historical `0.1.0` M06 installer remains beside it with SHA-256
  `0c52a295c8966837a9a69cec518704c0a151e80a7cace087b071c708bf4c2dfb`.

## Verification baseline

For the M08 `0.1.1` checkpoint, Windows verification passed with Node 24.15.0,
npm 11.12.1, and Rust 1.95:

- `npm run check` - 0 errors and 0 warnings
- `npm run build` - production bundle built
- `npm run test:mcp` - 28 tools and all shared/failure-path suites passed
- `npm run test:mcp:stdio` - passed
- `npm run test:mcp:http` - passed against a separately running local server
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` - passed
- `cargo check --manifest-path src-tauri/Cargo.toml` - passed
- `cargo test --manifest-path src-tauri/Cargo.toml` - 22 tests passed
- `npm audit --audit-level=moderate` - 0 vulnerabilities

The separately hosted HTTP smoke passed with all 28 tools. Installed QA created
Mosscap Scout session `mosscap-scout-20260727225718-dc51655e`, displayed
request `concept-gen-r0001-20260727225718-749783a6`, restored both after a full
restart, and exposed the exact request through MCP. A live connected Codex
ImageGen capability fulfilled all three requested outputs separately; the MCP
gateway imported r1-r3 with generated provenance and the installed desktop
displayed every revision after restart.

Re-run checks relevant to every change. Run `npm run mcp:http` in a separate
terminal before `npm run test:mcp:http`.

## Current human gate

M08 remains complete. The user explicitly accepted Mosscap Scout Turnaround r1,
rejected feet-only Walk Cycle r1, and requested a slight walking bob. Immutable
Walk Cycle `walk-cycle-r0002-20260727233042-abdbb047` adds that one-pixel bob
on frames 1 and 3 and now waits at the motion/readability gate.

Present the animated and static four-direction evidence from
`.studio\mosscap-r2-walk` without controlling the user's desktop. Do not create
a World Test until the user explicitly accepts motion and readability for this
exact Walk Cycle r2. If accepted, continue through the existing unchanged
workflow:

```text
accepted Walk Cycle -> World Test -> approved final art -> draft Export
```

This gate does not authorize final-art approval, publishing, a provider API,
additional paid services, or broader version 1 scope. Keep the Orc Vanguard
draft and all Mosscap Scout revisions immutable.

## Handoff discipline

When completing a milestone:

1. update this file's source, release, implemented, and next-milestone truth;
2. update `docs/ROADMAP.md`;
3. record durable choices in `docs/DECISIONS.md`;
4. update `docs/START_NEW_CHAT.md`;
5. run and report the relevant checks;
6. preserve unrelated user changes and every immutable artifact;
7. commit and push only when the user has authorized it.

The current user has authorized coherent verified commits and pushes for this
project. Still report the exact commit and branch, and never treat that
authorization as permission to publish art or use a paid service.
