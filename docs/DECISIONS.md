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

## 2026-07-26 - Recomputed structural validation reports

M03 validation is a read-only deterministic projection of an immutable
candidate PNG, not mutable candidate state. Both adapters return the same
versioned report shape:

- validator, candidate, candidate SHA-256, and contract identities;
- seven canonically ordered rule results with Pass, Fail, or Not assessed;
- Pass, Fail, and Not assessed totals derived from those results;
- a separate visual-judgment record fixed to Not assessed with user authority.

Canvas dimensions, hard alpha, visible actor height, exact foot-anchor contact,
visible RGB palette count, and frame-edge contact are measured from decoded
pixels. Ground-luma separation reports Not assessed until a pinned ground
reference exists. A failure does not suppress later measurements.

Reports are recomputed and are not written into candidate directories. The
desktop and MCP adapters therefore cannot mutate the evidence they validate or
silently turn structural success into approval.

Reason: immutable input makes deterministic reports reproducible. Avoiding a
stored mutable status removes stale-report and accidental-promotion paths while
the candidate SHA-256 keeps every report tied to exact source bytes.

## 2026-07-26 - No incremental AI-service spend

Actor Studio may only use AI capabilities covered by the user's existing
subscriptions. Do not enable pay-as-you-go APIs, purchased credits,
usage-metered billing, or paid add-ons. If a provider's billing boundary is
unclear, stop before connecting it and ask the user.

Local deterministic operations, including M03 validation, require no external
AI provider and incur no additional service cost.

Reason: the studio must remain predictable to operate and must never create an
unexpected bill while moving work between supported AI clients.

## 2026-07-27 - Turnaround document is the durable Concept-selection receipt

M04 does not mutate a Concept or add an approval field to it. The first
Turnaround revision records the selected Concept id and SHA-256,
`selectedBy: user`, and selection time. Its down PNG must match the selected
Concept byte for byte.

Reason: selection for the next stage is not final-art approval, but it must
still be durable and auditable. Binding the Turnaround to exact source bytes
prevents a later agent from silently repairing or replacing the user's choice.

## 2026-07-27 - Four-view Turnarounds are immutable atomic artifacts

Each Turnaround revision contains one versioned `turnaround.json` plus original
`down.png`, `right.png`, `up.png`, and `left.png` files in canonical contract
order. The TypeScript/MCP and Rust/Tauri adapters share the document, identity
rules, hash verification, atomic-directory publish behavior, and compatibility
fixture.

Structural validation is recomputed independently for each direction and
aggregated. Identity consistency remains Not assessed with user authority; no
agent operation can accept the Turnaround or authorize final art.

Reason: four loose candidate files could be mixed across repairs or identities.
One atomic immutable artifact keeps a coherent comparison set while preserving
the human gate before animation.

## 2026-07-27 - Subscription image generation remains outside the core

The first real Concept and Turnaround sources were prepared with OpenAI's
built-in ImageGen covered by the user's active subscription. Provider prompts
and raw sources remain in ignored `.studio/` work state, while the shared core
stores only provider-neutral provenance and immutable PNG evidence.

No API key, pay-as-you-go endpoint, purchased credit, usage billing, or paid
add-on is integrated.

Reason: this permits real artwork without coupling the studio protocol to one
client or creating incremental AI-service spend.

## 2026-07-27 - Direction repairs preserve unaffected Turnaround bytes

When the user rejects one direction, the repair still publishes a new complete
atomic Turnaround revision. Unaffected direction PNGs are reused byte for byte,
the rejected direction receives a new source and SHA-256 identity, and all
earlier Turnarounds remain available for comparison.

The first application is Mirelight Pilgrim r2: the user identified the r1 right
outline as bleeding and accepted the left profile as the better reference. R2
therefore replaces only the right view with a deterministic mirrored,
foot-anchor-corrected copy of the left view. Down, up, and left retain their
exact r1 hashes. The repair uses no AI service and does not imply user approval.

Reason: a narrow correction should not introduce silent drift into directions
the user did not reject. Publishing the complete repaired set atomically keeps
the stage coherent while retaining a precise audit trail from feedback to
changed bytes.

## 2026-07-27 - Walk Cycle is the durable accepted-Turnaround receipt

M04 does not mutate a Turnaround or add an approval field to it. The first Walk
Cycle revision records the exact accepted Turnaround id, its four direction
hashes and byte lengths, `acceptedBy: user`, and acceptance time. Frame 0 in
every direction must preserve the corresponding accepted Turnaround PNG bytes.

Reason: user acceptance unlocks animation but is not final-art approval.
Binding the next-stage artifact to exact source identities makes that
transition durable without creating an agent-writable approval state.

## 2026-07-27 - Sixteen-frame Walk Cycles are immutable atomic artifacts

Each Walk Cycle revision contains one versioned `walk-cycle.json` plus four
original PNG frames for each canonical down/right/up/left direction. The fixed
clip is `walk`, the duration is 300 ms, and files use direction-major frame 0–3
order. TypeScript/MCP and Rust/Tauri adapters share the document, hash
verification, atomic-directory publish behavior, and compatibility fixture.

Structural validation is recomputed across all sixteen PNGs and aggregated.
Motion and readability remain Not assessed with user authority; no agent
operation can accept the animation, approve final art, or publish it.

Reason: a complete atomic 4 × 4 artifact prevents mixed animation revisions,
keeps the accepted poses auditable, and gives the desktop and every agent
client the same motion-review evidence.

## 2026-07-27 - World Test uses a copied and SHA-256-pinned TileForge subset

Actor Studio tracks `reference-packs/tileforge-world-test-v1` as its only
TileForge input. The pack contains Scale Lineup, Forest Clearing, Crownhold,
and Tidewater in forest, autumn, dusk, and winter at ground-truth 1x scale.
Its manifest records the source checkout commit, the generated-engine commit,
source dimensions, byte lengths, SHA-256 identities, fixed viewports, actor
placements, and ground samples.

The upstream copyright notice is preserved. Actor Studio imports no TileForge
runtime code and never writes to the source repository.

Reason: a copied versioned pack makes visual evidence reproducible and lets
desktop and MCP clients verify exact inputs without coupling either product's
runtime or mutable working tree.

## 2026-07-27 - World Test is the durable accepted-Walk-Cycle receipt

The first World Test revision records the exact Walk Cycle id plus all sixteen
frame hashes and byte lengths with `acceptedBy: user`. It also records the
reference-manifest hash and upstream commit identities. This transition
captures the user's motion/readability acceptance but is not final-art
approval.

The local deterministic compositor places accepted down frame 0 into every
pinned scene/theme and atomically preserves sixteen immutable 640 x 384 PNGs
with `world-test.json`. The preparation record explicitly states that no
additional AI cost was incurred.

Reason: World Test must not silently swap animation frames or mutable
backgrounds. One atomic receipt keeps the final visual review tied to exact
actor and world evidence without adding a paid service.

## 2026-07-27 - Ground contrast resolves as a World Test measurement

Concept, Turnaround, and Walk Cycle structural reports continue to return
ground luma as Not assessed because they have no pinned placement. World Test
resolves the rule by comparing each frame's rounded mean visible-pixel luma
with each manifest-defined ground-sample mean: sixteen frames times sixteen
references, or 256 ordered measurements. A distance of at least the contract
minimum 15 passes.

Mean luma is a deterministic screening proxy, not a visual verdict. The report
retains a separate final-art judgment fixed to Not assessed with user
authority. Mirelight Pilgrim r1 reports 240 Pass and 16 Fail; only dusk grass
in Scale Lineup and Forest Clearing produces failures.

Reason: resolving the deferred structural rule at the first stage with real
ground context keeps earlier validators honest, surfaces camouflage risk, and
does not let arithmetic replace user judgment.

## 2026-07-27 - Export is the durable final-art approval receipt

World Test documents remain immutable and retain final-art judgment as Not
assessed. After the user explicitly approves one exact World Test, the next
Export revision records its document SHA-256, all sixteen preview identities,
`approvedBy: user`, and approval time, plus the exact sixteen Walk Cycle source
identities.

Reason: the final-art decision must be durable without adding an
agent-writable approval field to an earlier artifact. Binding the transition to
exact document and image identities prevents a different World Test or frame
set from being exported under the user's decision.

## 2026-07-27 - Version 1 Export is an immutable local draft package

Each Export revision atomically preserves:

```text
exports/<export-id>/
  export.json
  sprite-sheet.png
  metadata.json
  provenance.json
```

The 128 x 128 RGBA sheet uses down/right/up/left rows and frame 0–3 columns.
Metadata carries the contract id, actor identity, 32 px cell geometry, 300 ms
timing, foot anchor, coordinates, and source hashes. Provenance repeats the
user-approved World Test receipt, Walk Cycle receipt, local deterministic
preparation method, and `additionalAiCost: false`.

Desktop and MCP validation rehash all package files, reconstruct all sheet
pixels from the immutable Walk Cycle, and compare metadata and provenance.
Every package remains `draft`; publishing is fixed to `not_approved` with user
authority, and no publishing operation exists.

Reason: one small engine-neutral package completes the Brief-to-Export
workflow without coupling Actor Studio to a game runtime, paid service, or
publishing destination. Keeping publishing as a distinct absent capability
preserves the user's second approval gate.
