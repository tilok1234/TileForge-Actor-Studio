# Decision Register

This file records durable product and architecture decisions. Add a dated entry
when a choice changes the workflow or data boundary.

## 2026-07-26 - Narrow actor-only product

Version 1 creates one 32 px mob or NPC at a time. Tiles, maps, bosses, effects,
paperdolls, equipment, batching, and publishing are out of scope.

Reason: the previous general animation workflow grew incoherent. A narrow
product makes its stages, validation, and approval gates understandable.

## 2026-07-26 - Tauri 2 with Svelte and TypeScript

The desktop shell uses Tauri 2. The UI and shared domain helpers use Svelte 5
and TypeScript.

Reason: this provides a lightweight native Windows application while retaining
fast UI iteration and portable TypeScript for the agent-facing domain.

## 2026-07-26 - One client-neutral MCP gateway

Codex, Claude Code, and Antigravity connect to the same MCP surface. The gateway
supports stdio and localhost Streamable HTTP.

Reason: the studio contract must not drift into three client-specific
integrations.

## 2026-07-26 - Contract-first art boundaries

`contracts/tileforge-actor-32-v1.json` is the portable source of truth for
frame, art, animation, and approval rules.

Reason: creative prompts may vary, but the world boundary must be deterministic
and versioned.

## 2026-07-26 - Immutable candidates and human approval

Generated candidates are revisions, not overwrites. Agents cannot approve final
art, promote an approved candidate, or publish.

Reason: visual acceptance is a user decision, and iteration history is valuable
diagnostic evidence.

## 2026-07-26 - External repositories are read-only references

`animation_editor_live` and `Semantic tile generator design` are not modified
or imported as runtime dependencies.

Reason: the new application must remain clean, coherent, and independently
versioned.

## 2026-07-26 - Shared filesystem session protocol

The shared persistence boundary is a versioned local filesystem protocol, not
an MCP transport or a second desktop-only store. Both the Tauri backend and MCP
gateway read and publish:

```text
.studio/
  sessions/
    <session-id>/
      session.json
      candidates/
```

`TFAS_WORKSPACE` may redirect the `.studio` root. Session identity, document
shape, actor-brief limits, and the directory layout are compatibility rules.
TypeScript owns the client-neutral brief and session schemas; the Rust command
enforces the same rules at the native trust boundary. Both adapters are tested
against `tests/fixtures/session-v1.json`.

Creation is transactional at directory visibility: an adapter writes
`session.json` and `candidates/` inside a hidden same-parent temporary directory,
then renames it to the immutable final session id. A validation, write, or
identity-collision failure removes the temporary directory and never overwrites
an existing session.

Reason: requiring the desktop to connect to a separately running MCP server
would weaken standalone operation and packaging. Thin adapters over one local
protocol keep the store shared, local-first, and client-neutral while
compatibility tests make cross-language drift visible.

## 2026-07-26 - Immutable original PNG candidate protocol

Concept candidates extend the shared filesystem protocol without introducing a
generation provider:

```text
.studio/
  sessions/
    <session-id>/
      session.json
      candidates/
        <candidate-id>/
          candidate.json
          source.png
```

Each candidate has a versioned document, immutable id and revision, session and
contract identity, stage and direction, source provenance, byte length,
decoded dimensions, SHA-256 digest, intake evidence, and an `unreviewed`
review status. The source kind may be `imported` or `generated`; generated
provenance requires a provider name, but no provider adapter is part of M02.

Creation uses a hidden same-parent temporary directory followed by a rename.
Existing candidate identities are collisions, never replacement targets.
Readers rehash `source.png` before returning it. TypeScript and Rust adapters
share `tests/fixtures/concept-candidate-v1.json` so either can create or read
the same record.

M02 intake only establishes that the file is a decodable PNG, exactly 32 x 32,
within the local size limit, and contains transparency. It records structural
intake passes while keeping `reviewStatus` equal to `unreviewed` and visual
judgment equal to Not assessed. Hard-alpha, palette, actor bounds, foot anchor,
luma separation, and clipping are M03 validators.

Reason: retaining exact source bytes and evidence makes revisions comparable
and auditable across desktop and MCP clients. Separating intake from visual
acceptance prevents successful parsing from becoming accidental approval.
