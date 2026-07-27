# Claude Code Instructions

Use `AGENTS.md` and `docs/AGENT_WORKFLOW.md` as the canonical project
instructions. Read `contracts/tileforge-actor-32-v1.json` before sprite work.

Connect to the project MCP gateway described in `docs/MCP.md`. Do not invent
Claude-only mutations or approval behavior: final art approval is human-only.
When a Concept generation request exists, use its exact prompt only with
Claude's included native image capability if available in the active task;
otherwise leave it durable for another client or manual import. Creating the
request does not invoke Claude automatically. Never add a paid API or
credential.
