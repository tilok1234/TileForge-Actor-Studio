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
- generate or import a new candidate when that capability is implemented
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
4. Call `import_concept_candidate` to publish each new PNG as an immutable
   revision.
5. Use `list_concept_candidates` and `get_concept_candidate` to compare saved
   evidence.
6. Run validators and present their evidence.
7. Ask the user for visual approval at the defined gates.

The MCP tools currently implement steps 1–5. M02 candidate intake proves PNG
structure, exact dimensions, and the presence of transparency; it does not
perform the M03 validator suite or visual acceptance. Validation and export
tools will be added as their studio layers are built.

## Validation language

Use these distinct outcomes:

- **Pass** — a deterministic rule was measured and satisfied.
- **Fail** — a deterministic rule was measured and violated.
- **Not assessed** — the artifact or required view is unavailable.
- **Visually acceptable** — human judgment, never inferred from a structural
  pass.

## Local state

Sessions live in `.studio/sessions` by default. Set `TFAS_WORKSPACE` to redirect
local state. The desktop and MCP gateway read the same immutable session and
candidate documents; creating either publishes a complete directory atomically
rather than exposing a partial record. Original candidate PNG bytes are
rehash-verified on read and never overwritten. `.studio/` and generated exports
are ignored by Git.
