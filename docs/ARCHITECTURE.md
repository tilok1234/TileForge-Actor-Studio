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

`src/lib/studio/` contains client-neutral types, session, Concept, and
Turnaround documents, intake rules, versioned validation-report schemas,
deterministic PNG validators, and prompt compilation. Business rules belong
here.

### MCP gateway

`mcp/` exposes the shared core over standard MCP transports. It owns no art
style rules and grants no final-approval capability. Its adapter reads and
publishes the same session, Concept, and Turnaround documents as the desktop
backend and exposes the shared read-only validators.

### Desktop shell

`src/` renders the focused workflow. Tauri 2 in `src-tauri/` supplies the native
Windows shell and the desktop adapter for durable session, Concept, and
Turnaround commands. Rust independently enforces the same validator semantics
at the native trust boundary. The UI shows structural evidence while remaining
the eventual human approval surface.

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

Concept directories contain `candidate.json` and the original `source.png`.
A complete session, Concept, or Turnaround is first written to a hidden
same-parent temporary directory, then published with one rename so readers
never observe a partial record. `TFAS_WORKSPACE` redirects the root for either
adapter and for tests.

The JSON documents, identity rules, brief and intake limits, hashes, and
directory layout form one local protocol; the Rust and TypeScript adapters are
not separate stores. `tests/fixtures/session-v1.json`,
`tests/fixtures/concept-candidate-v1.json`, and
`tests/fixtures/turnaround-candidate-v1.json` are read by both test suites to
detect storage drift. `tests/fixtures/validation-report-v1.json` guards the
recomputed cross-language report shape and semantics. `.studio/` is not
source-controlled.

### Structural validation

M03 reports are deterministic projections of immutable candidate PNG bytes.
They are keyed by candidate id, candidate SHA-256, contract id, and validator
version but are not stored in `.studio/`. Desktop and MCP clients recompute the
same seven ordered rule results. Six rules inspect decoded pixels; ground luma
returns Not assessed until a pinned reference pack supplies a real comparison.

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

## Reference boundary

TileForge remains a separate project. Actor Studio will eventually consume a
pinned, copied reference pack with provenance; it will not import arbitrary
runtime code or write back into TileForge. The old animation editor is design
evidence only, not a dependency.
