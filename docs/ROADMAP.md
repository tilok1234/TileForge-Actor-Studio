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

Status: next.

Goal: accept one generated or imported down-facing concept while remaining
provider-neutral.

Acceptance criteria:

- provider adapters are outside the studio contract/core;
- each candidate receives an immutable id and provenance record;
- original image bytes are never overwritten;
- the UI can compare candidates at 1x, 8x, and 16x;
- candidate creation cannot imply approval;
- malformed dimensions, alpha, or file types fail safely.

## M03 - Contract Validation

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

## M04 - Turnaround and Walk Cycle

Goal: move an accepted concept through consistent four-direction views and a
four-frame walk cycle per direction.

- down, right, up, left order
- four frames per direction
- 300 ms default frame duration
- identity and scale consistency checks
- immutable revisions at every repair

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
