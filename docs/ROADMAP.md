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
- foot-anchor contact
- palette maximum
- minimum ground luma separation
- no frame-edge clipping

Reports must distinguish Pass, Fail, Not assessed, and human visual acceptance.

Implemented:

- one versioned, candidate-hash-bound report schema is shared by desktop and
  MCP adapters;
- decoded PNG pixels, rather than candidate metadata, drive canvas, hard-alpha,
  visible-height, foot-anchor, visible-palette, and frame-edge measurements;
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

Status: in progress. Turnaround is implemented and awaiting the user's
identity-consistency decision. Walk Cycle has not started.

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
- shared TypeScript and Rust compatibility/failure-path tests cover the
  document, exact-down invariant, malformed views, immutable reads, collisions,
  and partial-write cleanup.

Pending after the human Turnaround gate:

- explicit user acceptance of identity consistency;
- four walk frames per direction at the contract's 300 ms default;
- immutable Walk Cycle revisions and motion/readability review.

## M05 - World Test and Export

Goal: show the actor in pinned TileForge reference scenes and produce a
reviewable export package.

- copied, versioned reference pack with provenance
- scale and ground-readability previews
- PNG sheet, metadata, contract id, and provenance
- explicit user approval before promotion
- publishing remains a separate explicit action

## Deferred beyond version 1

- tiles and map editing
- bosses and large formats
- attacks, deaths, and skill effects
- paperdolls and equipment
- batch generation
- autonomous approval
- automatic publishing
