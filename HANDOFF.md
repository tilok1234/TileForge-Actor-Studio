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
- expected remote: `https://github.com/tilok1234/TileForge-Actor-Studio.git`
- a later documentation-only audit commit may be at HEAD
- expected state after the audit: clean and synchronized with `origin/main`
- source milestone: M07 complete
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

## Implemented through M07

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

## Source versus installed release

This distinction is important:

- Source `main` contains M07 from implementation commit `81f443e`.
- All package version fields are still `0.1.0`.
- The existing installer
  `src-tauri\target\release\bundle\nsis\TileForge Actor Studio_0.1.0_x64-setup.exe`
  was built for the M06 release-hardening checkpoint before M07.
- Its recorded SHA-256 is
  `0c52a295c8966837a9a69cec518704c0a151e80a7cace087b071c708bf4c2dfb`.
- Fresh-install and installed-restart QA proved M06 storage and Export restore,
  but it does not prove that an installed app contains the M07 generation
  request UI or behavior.

Do not call `0.1.0` the current M07 installer. The next release proof must
produce and test a new versioned package.

## Verification baseline

At commit `81f443e`, Windows verification passed with Node 24.15.0,
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

Native M07 QA used an isolated workspace and proved that Brief -> Concept
created session `mirelight-pilgrim-20260727073811-e6c44d8a`, displayed request
`concept-gen-r0001-20260727073811-33734fa0`, requested three separate
candidates, showed user-only approval, and restored the durable request.

Re-run checks relevant to every change. Run `npm run mcp:http` in a separate
terminal before `npm run test:mcp:http`.

## Recommended next milestone: M08

M08 is an installed cross-client generation proof, not a new provider
integration.

1. Bump the application package consistently from `0.1.0` to `0.1.1` in
   `package.json`, the root `package-lock.json` entries,
   `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and the Actor Studio
   package entry in `src-tauri/Cargo.lock`.
2. Build a new current-user NSIS installer from the verified M07 source.
3. Install it and prove that the installed desktop creates a generation
   request, displays its stable identity, and restores it after a full app
   restart.
4. Verify that MCP reads the exact same installed-workspace request.
5. Create one fresh, simple actor session for the cross-client proof.
6. In an active connected client, read the newest request and confirm whether
   that client actually has an included native image capability.
7. If it does, generate the requested outputs separately and import each as an
   immutable unreviewed Concept revision. If it does not, retain the request
   and report the limitation; do not connect a paid API.
8. Show the imported candidates in the installed desktop and stop for the
   user's visual selection.

M08 must not claim that request creation automatically wakes or controls an AI
client. It must not add provider credentials, usage billing, autonomous
approval, publishing, or broader art scope.

The completed Orc Vanguard draft remains immutable during this proof.

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
