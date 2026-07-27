# Shared Agent Workflow

Codex, Claude Code, and Antigravity are three clients of one studio contract.
They should not contain separate interpretations of the art rules.

## Authority

The user owns:

- final visual approval
- contract changes
- promotion of a candidate into an approved export
- publishing and destructive cleanup

An agent may:

- read the contract and sessions
- turn a creative brief into a constrained prompt
- create a local session
- generate a new candidate with a connected client's native subscription
  image capability, or import a local candidate
- compare candidates
- run structural and visual validators
- prepare a draft export

An agent may not:

- mark a candidate approved
- replace or rewrite an older candidate
- weaken constraints to make a validation pass
- alter source reference repositories
- publish an export

## Stage contract

| Stage | Input | Agent output | Human gate |
| --- | --- | --- | --- |
| Brief | Name, mob/NPC, short description | Compiled constrained prompt | Brief remains editable |
| Concept | Compiled prompt | Immutable front-view candidates | Select a concept |
| Turnaround | Selected concept | Down/right/up/left views | Accept identity consistency |
| Animate | Accepted turnaround | Four walk frames per direction | Accept motion/readability |
| World test | Candidate sheet | Scale and terrain previews plus validator report | Approve final art |
| Export | Approved candidate | PNG, metadata, provenance | Approve publishing separately |

Later stages cannot silently repair an earlier rejected stage. They create a new
revision and preserve the evidence trail.

## Contract-first sequence

1. Call `get_studio_contract`.
2. Call `compile_actor_prompt` with the creative brief.
3. Call `create_sprite_session` once the user wants persistent local work.
4. Call `create_concept_generation_request`, or read the newest request from
   `list_concept_generation_requests`, before using image generation. Use the
   request's exact prompt separately for each requested output.
5. Use only the connected client's native image capability covered by the
   user's subscription. If that capability is unavailable, leave the request
   intact and use manual PNG import; do not substitute a paid API.
6. Call `import_concept_candidate` to publish each new PNG as an immutable
   revision.
7. Use `list_concept_candidates` and `get_concept_candidate` to compare saved
   evidence.
8. Call `validate_concept_candidate` and present every Pass, Fail, and Not
   assessed result without converting it into visual acceptance.
9. After the user explicitly selects a Concept, call
   `create_turnaround_candidate` with the exact selected PNG as `down` plus
   right, up, and left views.
10. Use `list_turnaround_candidates`, `get_turnaround_candidate`, and
   `validate_turnaround_candidate` to compare and measure immutable Turnaround
   revisions.
11. Ask the user to accept or reject identity consistency across the four views.
12. Do not begin Walk Cycle work until that Turnaround gate is explicit.
13. After the user explicitly accepts a Turnaround, call
    `create_walk_cycle_candidate` with four frames per canonical direction.
    Frame 0 must be the exact accepted Turnaround PNG.
14. Use `list_walk_cycle_candidates`, `get_walk_cycle_candidate`, and
    `validate_walk_cycle_candidate` to compare and measure immutable animation
    revisions.
15. Ask the user to accept or reject motion and readability.
16. Do not begin World Test work until that Walk Cycle gate is explicit.
17. After the user explicitly accepts a Walk Cycle, call
    `create_world_test_candidate` to record the exact sixteen-frame source
    receipt and prepare all sixteen pinned scene/theme previews locally.
18. Use `list_world_test_candidates`, `get_world_test_candidate`, and
    `validate_world_test_candidate` to compare immutable previews and the 256
    frame-to-ground luma measurements.
19. Ask the user to approve or reject final art. Do not prepare an export until
    that gate is explicit, and do not treat a structural result as approval.
20. After the user explicitly approves one World Test as final art, call
    `create_export_candidate` to record that exact approval receipt and prepare
    the local PNG sheet, metadata, and provenance.
21. Use `list_export_candidates`, `get_export_candidate`, and
    `validate_export_candidate` to inspect immutable draft packages and verify
    every file against the approved sources.
22. Keep publishing separate. A draft Export is not published, and no agent
    operation may approve or perform publishing.

The MCP tools currently implement steps 1–22. The generation request is a
durable work order, not a provider call: the app owns its exact prompt, output
contract, cost boundary, and approval boundary while the connected client owns
the optional subscription-native image invocation. M02 candidate intake proves
PNG structure, exact dimensions, and the presence of transparency. M03 validation
then measures the immutable decoded pixels. The Turnaround slice of M04 records
the user-selected Concept and requires its exact bytes as the down view before
atomically publishing the four directions. Ground luma remains Not assessed
until a pinned ground reference exists; visual and identity acceptance remain
user-only decisions. The Walk Cycle slice records the user's accepted
Turnaround receipt, requires exact frame-zero bytes for all four directions,
and atomically preserves sixteen frames at 300 ms. Concept and Turnaround art
must contact the exact `(16, 28)` placement anchor; Walk Cycle frames may
contact anywhere on row 28 so either foot can lift fully. Motion and
readability remain user-only. The World Test slice binds the accepted Walk
Cycle to a copied SHA-256-pinned TileForge pack, atomically preserves sixteen
previews, and resolves ground luma for every frame/reference pairing. Final-art
judgment stays Not assessed inside the World Test document; the next-stage
Export records the user's explicit decision against that exact immutable
document. The Export package is prepared and validated locally without an AI
service, remains a draft, and keeps publishing not approved with user
authority.

## Validation language

Use these distinct outcomes:

- **Pass** — a deterministic rule was measured and satisfied.
- **Fail** — a deterministic rule was measured and violated.
- **Not assessed** — the artifact or required view is unavailable.
- **Visually acceptable** — human judgment, never inferred from a structural
  pass.

## Local state

On Windows, sessions live under
`%LOCALAPPDATA%\TileForge\Actor Studio\.studio\sessions` by default. Set
`TFAS_WORKSPACE` to redirect local state for both desktop and MCP; non-Windows
source development falls back to the ignored repository `.studio/`. The
desktop and MCP gateway read the same immutable session, generation-request,
Concept, Turnaround, Walk Cycle, World Test, and Export documents; creation
publishes complete directories atomically rather than exposing partial
records. Original PNG bytes are rehash-verified on read and never overwritten.
Draft Export
directories preserve `export.json`, `sprite-sheet.png`, `metadata.json`, and
`provenance.json`.

## Cost boundary

Use only AI capabilities covered by the user's existing subscriptions. Do not
connect pay-as-you-go APIs, buy credits, enable usage billing, or add paid
features. A generation request never stores API credentials and never calls a
provider itself. If a connected client cannot satisfy it with an included
native image tool, report that limitation and retain the request for another
client or manual import. Local contract validation does not require an AI
service.
