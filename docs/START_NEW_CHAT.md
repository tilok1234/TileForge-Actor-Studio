# New Chat Starter

Copy the prompt below into a new chat opened for the dedicated
`TileForge-Actor-Studio` project.

```text
Continue TileForge Actor Studio in the exact repository:

C:\Users\headc\Documents\TileForge-Actor-Studio

Begin with a read-only orientation. Verify:

Get-Location
git status -sb
git log -5 --oneline --decorate
git remote -v

Then completely read:

1. AGENTS.md
2. HANDOFF.md
3. README.md
4. contracts/tileforge-actor-32-v1.json
5. docs/ARCHITECTURE.md
6. docs/AGENT_WORKFLOW.md
7. docs/ROADMAP.md
8. docs/DECISIONS.md
9. docs/MCP.md
10. docs/START_NEW_CHAT.md

Reconcile the documentation with the live code, package versions, local
artifacts, and Git state before editing. Preserve existing dirty work and
report anything that disagrees with the handoff.

Hard boundaries:

- Do not modify C:\Users\headc\Documents\animation_editor_live.
- Do not modify
  C:\Users\headc\Documents\Semantic tile generator design.
- Treat both external repositories as read-only evidence.
- Keep version 1 focused on one 32 px mob or NPC at a time.
- Agents may create, compare, validate, and prepare candidates, but only I may
  select stage transitions, approve final art, or approve publishing.
- Never overwrite a generated candidate; create a new immutable revision.
- Keep Codex, Claude, and Antigravity on the same client-neutral MCP contract.
- Do not add maps, tiles, bosses, effects, attacks, equipment, paperdolls,
  batching, autonomous approval, or publishing.
- Do not use any AI service that costs extra beyond my existing subscriptions.
  No pay-as-you-go APIs, purchased credits, usage billing, or paid add-ons.

Expected checkpoint, which you must verify live:

- main contains M07 implementation commit 81f443e
  (Add subscription-native generation requests), documentation audit 0a54b33,
  and the containing M08 verification checkpoint;
- main is expected to be clean and synchronized with origin/main;
- M08 installed cross-client generation proof is complete;
- all application version fields are 0.1.1;
- MCP exposes 28 tools;
- desktop and MCP share immutable generation requests and later artifacts
  under %LOCALAPPDATA%\TileForge\Actor Studio\.studio by default;
- TFAS_WORKSPACE overrides that root for both adapters;
- current installer TileForge Actor Studio_0.1.1_x64-setup.exe has SHA-256
  b9686db13d425083a9c8d55cf558c2796dd18d7c40c6ccd19bfea1292a97ff96;
- installed tileforge-actor-studio.exe reports version 0.1.1 and SHA-256
  78e965b8c6adc392f385967043ca28156af1390097760c8cc6c59d92f374be5f;
- publishing remains absent and every Export remains a local draft.

Continue from the Mosscap Scout Walk Cycle motion/readability gate.

Installed proof evidence:

- session: mosscap-scout-20260727225718-dc51655e
- request: concept-gen-r0001-20260727225718-749783a6
- Concept r1: concept-r0001-20260727230409-aa483745
- Concept r2: concept-r0002-20260727230409-e9e46a75
- Concept r3: concept-r0003-20260727230409-b9b4f0de
- all three are immutable, generated, unreviewed, and report
  6 Pass / 0 Fail / 1 Not assessed
- the user explicitly selected Concept r1
- immutable Turnaround r1:
  turnaround-r0001-20260727231939-6d597fb3
- its down PNG is byte-identical to selected Concept r1
- Turnaround validation: 24 Pass / 0 Fail / 4 Not assessed
- the user explicitly accepted exact Turnaround r1 for animation
- immutable Walk Cycle r1:
  walk-cycle-r0001-20260727232551-4db453a0
- the user rejected r1 because only its feet moved; r1 remains immutable
- immutable Walk Cycle r2:
  walk-cycle-r0002-20260727233042-abdbb047
- r2 preserves the r1 foot motion and the accepted Turnaround PNG byte for byte
  as frames 0 and 2, while moving the upper body down one pixel on frames 1
  and 3
- timing: 300 ms; neutral / step / neutral / opposite step
- r2 Walk Cycle validation: 96 Pass / 0 Fail / 16 Not assessed
- motion/readability remains Not assessed with user authority
- no World Test is authorized

1. Verify the live Git state, `0.1.1` package fields, installer identity, and
   exact Mosscap Scout Concept, Turnaround, and Walk Cycle records before
   acting.
2. Work headlessly through MCP and local review artifacts. Do not use Computer
   Use or take control of my desktop unless I explicitly request UI-specific
   QA.
3. Present
   .studio\mosscap-r2-walk\walk-cycle-all-directions.gif and
   .studio\mosscap-r2-walk\walk-cycle-review.png in canonical
   down/right/up/left order.
4. Stop for my explicit motion/readability decision: accept exact Walk Cycle r2
   or reject it. Never infer acceptance from structural validation.
5. If I accept it, create a new immutable World Test through the existing
   contract and wait again for my final-art approval.
6. If I reject it, preserve it and wait for an explicit repair or new Walk
   Cycle request. Do not silently replace it.
7. Do not add a provider API, paid fallback, new scope, autonomous approval, or
   publishing.

The completed Orc Vanguard checkpoint must remain immutable:

- session: orc-vanguard-20260727012850-6bb50608
- accepted Walk Cycle:
  walk-cycle-r0011-20260727064901-0ebf795f
- approved World Test:
  world-test-r0001-20260727065711-23d42e04
- draft Export:
  export-r0001-20260727070603-f9fe69a3
- publishing: not_approved

The current blocker is intentionally my visual authority, so stop after
presenting Walk Cycle r2 unless I have explicitly accepted or rejected its
motion/readability. After explicit acceptance, make reasonable implementation
decisions within the unchanged workflow.

If source or package files change before claiming another milestone, run:

npm run check
npm run build
npm run test:mcp
npm run test:mcp:stdio
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm audit --audit-level=moderate

For the HTTP MCP smoke test, start npm run mcp:http separately and then run:

npm run test:mcp:http

Keep HANDOFF.md, docs/ROADMAP.md, docs/DECISIONS.md, and
docs/START_NEW_CHAT.md synchronized with any verified behavior change. You may
commit and push a coherent verified checkpoint; report the exact commit and
branch. That permission does not authorize publishing art or using a paid
service.
```
