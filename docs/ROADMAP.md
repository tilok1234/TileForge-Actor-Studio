# Roadmap

The roadmap is organized as narrow vertical slices. A milestone is complete
only when its user-visible behavior and verification evidence both exist.

## M00 - Foundation

Status: complete.

- Tauri 2 and Svelte 5 application shell
- Six-stage focused workflow UI
- Versioned TileForge 32 px actor contract
- Shared prompt compiler and session types
- MCP server with stdio and Streamable HTTP transports
- Durable MCP-created session JSON
- Codex, Claude Code, and Antigravity guidance
- Human-only approval boundary
- Build, type, transport, Rust, browser, and audit checks

## M01 - Durable Brief to Concept

Status: complete.

Goal: make the first workflow transition real and shared without generating art.

Implemented:

- `.studio/sessions/<id>` is the documented filesystem protocol used by both
  desktop and MCP adapters;
- one shared TypeScript brief schema supplies UI and MCP input limits, with the
  Rust command enforcing the same limits at its trust boundary;
- Begin concept atomically publishes `session.json` and `candidates/`, then
  advances to Concept;
- the UI displays session id, revision, contract id, and local saved state;
- desktop startup reopens the newest valid session;
- invalid input and identity collisions fail without partial session
  directories or overwrites;
- a shared session fixture and focused TypeScript/Rust tests prove document
  compatibility;
- sessions contain no approval field and no agent/backend approval operation
  exists.

## M02 - Immutable Concept Candidates

Status: complete.

Goal: accept one generated or imported down-facing concept while remaining
provider-neutral.

Acceptance criteria:

- provider adapters are outside the studio contract/core;
- each candidate receives an immutable id and provenance record;
- original image bytes are never overwritten;
- the UI can compare candidates at 1x, 8x, and 16x;
- candidate creation cannot imply approval;
- malformed dimensions, alpha, or file types fail safely.

Implemented:

- provider-neutral local PNG intake is exposed through desktop and MCP
  adapters; no generation provider is selected or integrated;
- each candidate is atomically published under its own never-overwritten
  revision directory with a versioned document and original `source.png`;
- provenance, byte length, decoded dimensions, SHA-256 identity, and
  `unreviewed` status are shared compatibility fields;
- reads rehash the original bytes and fail if stored evidence no longer
  matches;
- the UI restores, lists, switches, and previews revisions at 1x, 8x, and 16x;
- malformed PNG data, non-32 x 32 images, fully opaque images, missing provider
  provenance, identity collisions, and partial writes fail safely;
- TypeScript and Rust use the same candidate fixture and exercise byte
  preservation, compatibility, and failure cleanup.

## M03 - Contract Validation

Status: complete.

Goal: measure structural rules before asking for visual judgment.

Initial validators:

- 32 x 32 canvas
- hard alpha
- actor height range
- exact static foot-anchor contact and animated walk ground-row contact
- palette maximum
- minimum ground luma separation
- no frame-edge clipping

Reports must distinguish Pass, Fail, Not assessed, and human visual acceptance.

Implemented:

- one versioned, candidate-hash-bound report schema is shared by desktop and
  MCP adapters;
- decoded PNG pixels, rather than candidate metadata, drive canvas, hard-alpha,
  visible-height, foot-anchor, visible-palette, and frame-edge measurements;
- Concept and Turnaround validation require exact contact at `(16, 28)`;
  animated Walk Cycle validation requires visible foot contact anywhere on
  contract row 28 so alternating feet can fully lift;
- independent rules continue running after another rule fails, preserving the
  full repair list;
- ground-luma separation reports Not assessed until a pinned ground reference
  exists;
- every report includes a separate user-owned visual-judgment field that
  remains Not assessed and cannot encode approval;
- validation is read-only, deterministic, local, and does not modify candidate
  bytes or directories;
- TypeScript and Rust use the same report fixture and exercise passing,
  multi-failure, decoded-dimension, immutability, and approval-boundary cases;
- the desktop displays per-rule evidence and Pass/Fail/Not assessed totals and
  can re-run the local validator for the selected immutable revision.

## M04 - Turnaround and Walk Cycle

Status: complete. The user accepted Turnaround r2 and Walk Cycle r1 for their
respective next-stage transitions. Neither decision is final-art approval.

Goal: move an accepted concept through consistent four-direction views and a
four-frame walk cycle per direction.

- down, right, up, left order
- four frames per direction
- 300 ms default frame duration
- identity and scale consistency checks
- immutable revisions at every repair

Turnaround implemented:

- one immutable Turnaround revision records the exact user-selected Concept id,
  SHA-256, user authority, and selection time;
- the down view must preserve the selected Concept PNG byte for byte;
- down, right, up, and left PNGs use canonical order and filenames and publish
  atomically under one never-overwritten revision directory;
- desktop and MCP adapters create, list, read, rehash, and structurally
  validate the same four-view document and original PNG bytes;
- the desktop restores the latest Turnaround after restart, displays the
  selection receipt and four-view comparison, and reports per-direction plus
  aggregate structural evidence;
- structural validation is recomputed, not stored, and identity consistency
  remains Not assessed with user authority;
- direction-specific feedback creates a new complete Turnaround revision while
  preserving unaffected direction bytes; local r2 changes only the rejected
  right-facing view and retains r1;
- shared TypeScript and Rust compatibility/failure-path tests cover the
  document, exact-down invariant, malformed views, immutable reads, collisions,
  and partial-write cleanup.

Walk Cycle implemented:

- the first Walk Cycle document records the exact accepted Turnaround id and
  four source hashes, `acceptedBy: user`, and acceptance time;
- frame 0 for each direction must preserve the accepted Turnaround PNG bytes;
- static Concept/Turnaround art retains exact `(16, 28)` placement contact,
  while Walk Cycle frames may ground either foot anywhere on contract row 28;
- sixteen original PNGs publish atomically in canonical down/right/up/left and
  frame 0–3 order with a fixed 300 ms contract duration;
- desktop and MCP adapters create, list, read, rehash, and structurally
  validate the same immutable Walk Cycle document and frame bytes;
- desktop restart opens Animate, plays all four directions at 300 ms, shows the
  durable r2 acceptance receipt, and presents aggregate plus per-direction
  evidence;
- structural validation is recomputed across all sixteen frames, while motion
  and readability remain Not assessed with user authority;
- shared TypeScript and Rust compatibility/failure-path tests cover exact
  frame-zero preservation, malformed frame sets, immutable reads, collisions,
  and partial-write cleanup.

## M05 - World Test and Export

Status: complete through a local draft Export. World Test r1 has explicit
user final-art approval for Export; publishing remains a separate unapproved
user gate.

Goal: show the actor in pinned TileForge reference scenes and produce a
reviewable export package.

- copied, versioned reference pack with provenance
- scale and ground-readability previews
- PNG sheet, metadata, contract id, and provenance
- explicit user approval before promotion
- publishing remains a separate explicit action

World Test implemented:

- a tracked `tileforge-world-test-v1` pack copies four exact TileForge scenes
  across forest, autumn, dusk, and winter with upstream checkout,
  generated-engine, byte-length, dimension, and SHA-256 provenance;
- one immutable World Test revision records the exact accepted Walk Cycle id
  plus all sixteen source hashes and byte lengths with `acceptedBy: user`;
- the local deterministic compositor creates sixteen 640 x 384 previews
  without an AI service or additional cost, then atomically publishes them
  with `world-test.json`;
- desktop and MCP adapters create, list, read, rehash, and validate the same
  World Test protocol; desktop restart restores World Test and exposes all
  scene/theme combinations;
- validation compares mean visible-actor luma for all sixteen frames with all
  sixteen pinned ground samples, producing 256 measured Pass/Fail results and
  zero Not assessed ground results;
- the real Mirelight Pilgrim World Test r1 reports 240 Pass / 16 Fail / 0 Not
  assessed; all failures are on dusk grass in Scale Lineup and Forest
  Clearing, while final-art judgment remains Not assessed with user authority;
- shared TypeScript and Rust compatibility/failure-path tests cover the
  accepted-source receipt, pack identity, immutable previews, collisions,
  partial-write cleanup, 256 measurements, and approval boundary.

Export implemented:

- the user's explicit World Test r1 final-art decision is recorded only in the
  next-stage Export receipt with the exact World Test document hash, all
  sixteen preview identities, `approvedBy: user`, and approval time;
- one immutable draft Export atomically preserves `export.json`, a 128 x 128
  RGBA `sprite-sheet.png`, `metadata.json`, and `provenance.json`;
- sheet rows follow canonical down/right/up/left order and columns contain
  frames 0–3, with every cell pixel-identical to its immutable Walk Cycle
  source;
- metadata records actor identity, contract id, 32 px cells, 300 ms timing,
  foot anchor, frame coordinates, and source hashes; provenance binds the
  approved World Test, source Walk Cycle, no-cost local preparation, and the
  closed publishing gate;
- desktop and MCP adapters create, list, read, restore, and validate the same
  Export protocol; desktop restart restores the full stable Export id, sheet,
  7 Pass / 0 Fail / 0 Not assessed package evidence, and publishing lock;
- shared TypeScript and Rust compatibility/failure-path tests cover the
  fixture, missing source, immutable file reads, source and pixel
  reconstruction, metadata/provenance compatibility, collisions, tamper
  detection, partial-write cleanup, no-cost boundary, and user-only
  publishing authority;
- the real local Mirelight Pilgrim Export r1 is
  `export-r0001-20260727001749-a13580f4`; it remains `draft` and no publishing
  operation exists.

## M06 - Windows release hardening

Status: complete.

Goal: make the finished version 1 workflow safe and usable from an installed
Windows application without changing the art or publishing scope.

Implemented:

- desktop and MCP now default to the same uninstall-safe per-user workspace at
  `%LOCALAPPDATA%\TileForge\Actor Studio\.studio`;
- `TFAS_WORKSPACE` keeps highest precedence, while non-Windows source
  development retains the repository `.studio` fallback;
- a focused compatibility test locks the override, packaged Windows default,
  and repository-fallback resolution order;
- the Export panel exposes Open Folder, but the native command first re-reads
  the exact immutable package and refuses missing or tampered identities before
  launching Explorer;
- Rust positive and missing-Export tests prove that validation occurs before
  the OS folder launcher is called;
- Tauri builds a current-user NSIS installer without administrator rights;
- a fresh installation of version 0.1.0 launched from its installed path and
  restored Mirelight Pilgrim Export r1 from the shared per-user workspace;
- release hardening adds no AI provider, publishing operation, paid service, or
  incremental billing.

## Post-M06 - Second-actor generality run

Status: in progress at the user-owned World Test final-art gate.

Goal: exercise the unchanged version 1 workflow with a visually and
semantically different actor, exposing any remaining assumptions tied to the
first Mirelight Pilgrim mob without widening product scope.

Current evidence:

- durable mob session `orc-vanguard-20260727012850-6bb50608` is shared by
  desktop and MCP in the per-user workspace;
- three immutable down-facing Concept candidates were imported through MCP;
- all three report 6 Pass / 0 Fail / 1 Not assessed, with ground luma correctly
  deferred until World Test and visual judgment retained by the user;
- the user selected exact Concept A
  `concept-r0001-20260727013308-fd1e91fe`;
- immutable Turnaround `turnaround-r0001-20260727014752-775090c7` records that
  exact user-selection receipt, preserves the selected down PNG byte for byte,
  and adds strict right/up/left sources;
- Turnaround validation reports 24 Pass / 0 Fail / 4 Not assessed, while
  identity consistency remained user-owned;
- the user accepted exact Turnaround r1 for animation with "looks good";
- immutable Walk Cycle `walk-cycle-r0001-20260727015657-df64eb2e` records that
  exact accepted-Turnaround receipt and preserves frame 0 byte for byte in all
  four directions;
- the user rejected Walk Cycle r1 because its feet remained planted while the
  actor only wiggled back and forth; r1 remains immutable;
- replacement Walk Cycle `walk-cycle-r0002-20260727020559-adbbcf52` records the
  same exact accepted-Turnaround receipt and again preserves frame 0 byte for
  byte in every direction;
- the local deterministic r2 loop uses the contract timing and a compact
  neutral/first-foot/neutral/opposite-foot rhythm suited to a heavy
  armored actor, now with alternating lifted boots in down/up and a two-pixel
  forward/back foot arc in right/left;
- the user confirmed r2's foot motion read better and requested cleanup of its
  thin planted-heel spikes and isolated side pixels;
- cleanup Walk Cycle `walk-cycle-r0003-20260727021420-5e86e86e` preserves r2's
  foot motion, removes artificial arm-tip and torso/hip seam nubs, replaces the
  narrow planted heel with a connected three-pixel boot core, and keeps each
  final frame in one four-connected silhouette;
- the user requested more vertical bob after reviewing r3;
- heavier Walk Cycle `walk-cycle-r0004-20260727021942-2ebffe25` preserves r3's
  cleaned silhouette and foot motion, adds a one-pixel upper-body weight drop
  on both passing-foot poses, returns to the exact neutral on alternating
  frames, and keeps every frame in one four-connected silhouette;
- the user reported that the foot broke in some r4 frames;
- stable-ankle Walk Cycle `walk-cycle-r0005-20260727043425-305dceaf` preserves
  r4's weight drop, keeps each ankle/lower-leg column fixed and connected, and
  restricts movement to the bottom three foot rows while keeping every frame
  in one four-connected silhouette;
- the user reported that r5's side walk looked weird;
- alternating-side-foot Walk Cycle `walk-cycle-r0006-20260727044208-11ae5cca`
  preserves all eight r5 down/up frames byte for byte and replaces the
  stretched side boot with one planted half-foot plus one half-foot that lifts
  and advances by one pixel;
- the user reported that r6's feet were broken again; its split half-foot
  technique produced fragmented passing-pose silhouettes and was abandoned;
- rigid-foot Walk Cycle `walk-cycle-r0007-20260727060748-4bd86734` preserves
  the passing-pose weight drop while moving each animated boot or
  lower-leg-and-boot shape as one connected unit, with no independently moving
  foot fragments;
- the user reported that r7 no longer lifted a foot and read as a grounded
  shuffle;
- lifted-step Walk Cycle `walk-cycle-r0008-20260727061503-7be83825` raises the
  complete swing lower leg and boot one pixel in every passing pose, retains a
  sturdy stance foot at the contract anchor, and moves the side-view swing leg
  two pixels through the stride;
- the user reported that r8 still left one foot unmoving in front/back because
  both passing poses animated only the non-anchor boot;
- alternating front/back Walk Cycle
  `walk-cycle-r0009-20260727061941-a5c9bfc9` preserves all eight r8 side frames
  byte for byte, fully lifts the non-anchor boot in one passing pose, and moves
  the anchor-side boot into a connected heel-pivot/toe-off pose in the other;
- the user reported that r9 was almost correct but one right-foot pixel still
  appeared not to move;
- one-pixel cleanup Walk Cycle `walk-cycle-r0010-20260727062615-7a408516`
  reshapes the anchor heel as a connected two-by-two color-shifted pivot and
  changes the one coincident up-facing outline value at `(21, 27)` to the
  adjacent boot-ramp value;
- the user still saw a stationary right-foot pixel in r10; the exact
  `(16, 28)` opacity requirement was the remaining forced support pixel;
- the user explicitly approved a narrow contract clarification: Concept and
  Turnaround keep exact anchor contact, frame 0 keeps exact accepted source
  bytes, and animated Walk Cycle frames may contact anywhere on row 28;
- full alternating-foot-lift Walk Cycle
  `walk-cycle-r0011-20260727064901-0ebf795f` preserves all eight r10 side
  frames byte for byte, fully lifts the down/up left boot in frame 1, and
  raises and advances the complete right lower leg and boot in frame 3;
- frames 0 and 2 remain exact neutral beats; down/up frame 3 has transparent
  alpha at `(16, 28)` while the opposite boot remains grounded on row 28, and
  every r11 frame is exactly one four-connected visible component;
- Walk Cycle validation reports 96 Pass / 0 Fail / 16 Not assessed, while
  structural evidence remains separate from user-owned motion judgment;
- the user explicitly accepted exact Walk Cycle r11 motion/readability with
  "nice very good", unlocking World Test but not final art or publishing;
- immutable World Test `world-test-r0001-20260727065711-23d42e04` records the
  exact r11 id, all sixteen frame hashes, user authority, and acceptance time;
- the unchanged pinned TileForge reference pack produced sixteen immutable
  640 x 384 scene/theme previews through the local deterministic compositor;
- World Test validation reports 256 Pass / 0 Fail / 0 Not assessed across all
  frame-to-ground measurements, while final-art judgment remains Not assessed
  with user authority;
- generation used built-in subscription ImageGen outside the provider-neutral
  core; transparency preparation and validation were local and incurred no
  additional AI-service cost;
- the next transition is blocked only on the user's approval or rejection of
  exact World Test r1 final art; no Export may be created before that decision;
- the larger attack-animation reference was translated only into original
  32 px character language; attacks, weapons, and equipment remain outside the
  v1 workflow;
- the paused Snowberry Courier session and all three of its unselected
  Concepts remain preserved without mutation.

## Deferred beyond version 1

- tiles and map editing
- bosses and large formats
- attacks, deaths, and skill effects
- paperdolls and equipment
- batch generation
- autonomous approval
- automatic publishing
