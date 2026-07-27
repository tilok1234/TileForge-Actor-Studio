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
  (Add subscription-native generation requests), followed only by a possible
  documentation-audit checkpoint;
- main is expected to be clean and synchronized with origin/main;
- M07 is complete in source;
- MCP exposes 28 tools;
- desktop and MCP share immutable generation requests and later artifacts
  under %LOCALAPPDATA%\TileForge\Actor Studio\.studio by default;
- TFAS_WORKSPACE overrides that root for both adapters;
- the existing TileForge Actor Studio_0.1.0_x64-setup.exe is the older verified
  M06 package and does not contain M07;
- publishing remains absent and every Export remains a local draft.

Continue with M08: Installed cross-client generation proof.

1. Verify and bump the application version consistently from 0.1.0 to 0.1.1
   in package.json, the root package-lock.json entries,
   src-tauri/tauri.conf.json, src-tauri/Cargo.toml, and the Actor Studio package
   entry in src-tauri/Cargo.lock. Do not bulk-rewrite unrelated dependency
   versions.
2. Run the required source checks, then build a new current-user NSIS installer
   from the M07 source and record its SHA-256.
3. Install 0.1.1 and prove the installed desktop creates a Concept generation
   request, shows its stable request id, and restores it after a full restart.
4. Verify MCP reads that exact request from the installed per-user workspace.
5. Create one fresh simple actor for the cross-client proof. Do not mutate or
   promote the completed Orc Vanguard draft.
6. In an active connected client, list and read the newest request. A request
   is a durable handoff, not a dispatched job: creating one does not wake or
   control Codex, Claude, Antigravity, or an image provider.
7. Confirm whether that client actually has a native image capability included
   in my subscription. If yes, use the exact request prompt separately for each
   requested output and import every PNG through import_concept_candidate with
   generated provenance. If not, retain the request and report the limitation;
   do not connect a paid API.
8. Show the immutable unreviewed candidates in the installed desktop and stop
   for my visual selection.

The completed Orc Vanguard checkpoint must remain immutable:

- session: orc-vanguard-20260727012850-6bb50608
- accepted Walk Cycle:
  walk-cycle-r0011-20260727064901-0ebf795f
- approved World Test:
  world-test-r0001-20260727065711-23d42e04
- draft Export:
  export-r0001-20260727070603-f9fe69a3
- publishing: not_approved

Make reasonable implementation decisions within this scope and keep working.
Do not stop at a plan unless a choice materially expands the product, requires
new spending, or requires my visual/publishing authority.

Before claiming completion, run:

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

Update HANDOFF.md, docs/ROADMAP.md, docs/DECISIONS.md, and
docs/START_NEW_CHAT.md to match the verified result. You may commit and push a
coherent verified checkpoint; report the exact commit and branch. That
permission does not authorize publishing art or using a paid service.
```
