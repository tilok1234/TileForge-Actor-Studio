# New Chat Starter

Copy the prompt below into a new chat opened for the dedicated
`TileForge-Actor-Studio` project.

```text
Continue TileForge Actor Studio in the exact repository:

C:\Users\headc\Documents\TileForge-Actor-Studio

Start with a read-only orientation. Verify Get-Location, git status -sb, the
latest git log, and the configured remote. Then read these files completely:

1. AGENTS.md
2. HANDOFF.md
3. README.md
4. contracts/tileforge-actor-32-v1.json
5. docs/ARCHITECTURE.md
6. docs/AGENT_WORKFLOW.md
7. docs/ROADMAP.md
8. docs/DECISIONS.md
9. docs/MCP.md

Reconcile the documentation with the live code and Git state before editing.
Preserve any existing dirty work and report anything that disagrees with the
handoff.

Hard boundaries:

- Do not modify C:\Users\headc\Documents\animation_editor_live.
- Do not modify C:\Users\headc\Documents\Semantic tile generator design.
- Treat both as read-only references only.
- Keep version 1 limited to one 32 px mob or NPC at a time.
- Agents may create, compare, validate, and prepare candidates, but only I may
  approve final art or publishing.
- Never overwrite generated candidates; create immutable revisions.
- Keep Codex, Claude, and Antigravity on the same client-neutral MCP contract.

After orientation, verify the implemented M04 Turnaround and Walk Cycle
behavior against the live code and checks. The user selected real Concept r4,
rejected the Turnaround r1 right-facing outline, then explicitly accepted
immutable Turnaround r2 for animation. Local Walk Cycle r1 contains four frames
per direction at 300 ms and is awaiting the user-owned motion/readability gate.
Do not begin World Test work until that gate is explicit. Keep every repair
immutable, keep Pass/Fail/Not assessed separate from visual acceptance, and do
not grant final-art approval or publishing authority.

Do not use any AI service that adds cost beyond my existing subscriptions. No
pay-as-you-go APIs, purchased credits, usage billing, or paid add-ons.

Make reasonable implementation decisions within that scope and keep moving.
Do not stop at a plan unless a choice would materially expand the product or
requires my authority.

Before claiming completion, run the relevant checks:

npm run check
npm run build
npm run test:mcp
npm run test:mcp:stdio
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
npm audit --audit-level=moderate

For the HTTP MCP smoke test, start npm run mcp:http separately and then run
npm run test:mcp:http.

Update HANDOFF.md, docs/ROADMAP.md, and docs/DECISIONS.md to match the result.
Do not commit or push unless I explicitly ask.
```
