# Architecture

TileForge Actor Studio uses a small layered design.

```text
Codex ─┐
Claude ├── MCP gateway ── TypeScript adapter ─┐
Antigravity ┘                                 ├── .studio session protocol
Svelte UI ── Tauri commands ── Rust adapter ──┘
```

## Layers

### Contract

`contracts/tileforge-actor-32-v1.json` is the portable, versioned definition of
frame, art, animation, and approval rules. The TypeScript representation in
`src/lib/studio/contract.ts` must stay equivalent.

### Shared studio core

`src/lib/studio/` contains client-neutral types, session, Concept, Turnaround,
Walk Cycle, pinned-reference, and World Test documents, intake rules,
versioned validation-report schemas, deterministic PNG validators, and prompt
compilation. Business rules belong here.

### MCP gateway

`mcp/` exposes the shared core over standard MCP transports. It owns no art
style rules and grants no final-approval capability. Its adapter reads and
publishes the same session, Concept, Turnaround, Walk Cycle, and World Test
documents as the desktop backend and exposes the shared read-only validators.

### Desktop shell

`src/` renders the focused workflow. Tauri 2 in `src-tauri/` supplies the native
Windows shell and the desktop adapter for durable session, Concept, Turnaround,
Walk Cycle, and World Test commands. Rust independently enforces the same
validator semantics at the native trust boundary. The UI shows structural
evidence while remaining the eventual human approval surface.

### Local workspace

`.studio/` is the shared persistence boundary. Both adapters use
`.studio/sessions/<session-id>/session.json`, immutable Concept directories at
`candidates/<candidate-id>/`, and immutable Turnaround directories:

```text
turnarounds/<turnaround-id>/
  turnaround.json
  down.png
  right.png
  up.png
  left.png
```

Walk Cycle directories extend the same protocol:

```text
walk-cycles/<walk-cycle-id>/
  walk-cycle.json
  down-0.png ... down-3.png
  right-0.png ... right-3.png
  up-0.png ... up-3.png
  left-0.png ... left-3.png
```

World Test directories preserve the accepted Walk Cycle receipt and all pinned
scene/theme composites:

```text
world-tests/<world-test-id>/
  world-test.json
  scale-lineup-forest.png ... scale-lineup-winter.png
  forest-clearing-forest.png ... forest-clearing-winter.png
  crownhold-forest.png ... crownhold-winter.png
  tidewater-forest.png ... tidewater-winter.png
```

Concept directories contain `candidate.json` and the original `source.png`.
A complete session, Concept, Turnaround, Walk Cycle, or World Test is first
written to a hidden same-parent temporary directory, then published with one
rename so readers never observe a partial record. `TFAS_WORKSPACE` redirects
the root for either adapter and for tests.

The JSON documents, identity rules, brief and intake limits, hashes, and
directory layout form one local protocol; the Rust and TypeScript adapters are
not separate stores. `tests/fixtures/session-v1.json`,
`tests/fixtures/concept-candidate-v1.json`,
`tests/fixtures/turnaround-candidate-v1.json`, and
`tests/fixtures/walk-cycle-candidate-v1.json`, and
`tests/fixtures/world-test-candidate-v1.json` are read by both test suites to
detect storage drift. `tests/fixtures/validation-report-v1.json` guards the
recomputed cross-language report shape and semantics. `.studio/` is not
source-controlled.

### Structural validation

M03 reports are deterministic projections of immutable candidate PNG bytes.
They are keyed by candidate id, candidate SHA-256, contract id, and validator
version but are not stored in `.studio/`. Desktop and MCP clients recompute the
same seven ordered rule results. Six rules inspect decoded pixels; ground luma
returns Not assessed until World Test adds a pinned reference and placement.

The report has no approval field. Its separate visual-judgment record is fixed
to Not assessed with user authority, so a structural Pass cannot become visual
acceptance.

### Turnaround selection and validation

A Turnaround document is also the durable receipt for the user's Concept
selection. It records the selected Concept id and SHA-256, `selectedBy: user`,
and the selection time. The down PNG must have the exact selected Concept hash
and byte length, preventing an agent from silently repairing or replacing the
chosen source while entering M04.

The four original direction PNGs are rehash-verified on every read. Turnaround
validation recomputes the M03 structural report for each view and aggregates
the totals. Its separate identity-consistency judgment is fixed to Not assessed
with user authority. Structural success therefore cannot accept a Turnaround
or authorize Walk Cycle work.

### Walk Cycle acceptance and validation

A Walk Cycle document is the durable receipt for the user's accepted
Turnaround. It records the Turnaround id, all four exact direction hashes,
`acceptedBy: user`, and the acceptance time. Frame 0 for every direction must
match that accepted source hash and byte length, preventing animation work from
silently changing an approved pose.

All sixteen original frame PNGs are rehash-verified on every read. Walk Cycle
validation recomputes the M03 structural report for every frame and aggregates
the totals. Its motion/readability judgment is fixed to Not assessed with user
authority. Structural success therefore cannot accept animation, approve final
art, or authorize World Test work.

### World Test acceptance and ground evidence

A World Test document is the durable receipt for the user's accepted Walk
Cycle. It records the exact Walk Cycle id and all sixteen source hashes and byte
lengths with `acceptedBy: user`. Its reference receipt binds the copied pack
manifest, upstream checkout, and generated-engine commits.

The local compositor places accepted down frame 0 into four scenes across four
themes and atomically preserves the sixteen resulting 640 x 384 PNGs. Read
operations rehash every preview. Validation recomputes mean visible-actor luma
for all sixteen frames and compares it with each pinned ground sample: 256
deterministic measurements using the contract's minimum distance of 15. This is
structural evidence, not a visual verdict. Final-art judgment remains fixed to
Not assessed with user authority.

## Reference boundary

TileForge remains a separate project. Actor Studio consumes the copied,
versioned `reference-packs/tileforge-world-test-v1` subset with per-file
SHA-256 provenance; it imports no TileForge runtime code and never writes back.
The old animation editor is design evidence only, not a dependency.
