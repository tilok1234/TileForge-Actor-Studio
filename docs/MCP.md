# MCP setup

Actor Studio exposes one client-neutral MCP server. Start it from the repository
root:

```powershell
npm install
npm run mcp:http
```

The server listens only on localhost:

```text
http://127.0.0.1:7331/mcp
```

The shared HTTP transport is the preferred setup because Codex, Claude Code,
and Antigravity can use the same running gateway.

## Codex

Add the local endpoint:

```powershell
codex mcp add tileforge-actor-studio --url http://127.0.0.1:7331/mcp
codex mcp list
```

An equivalent TOML example is in `.codex/config.toml.example`. Do not copy it
into a user-level configuration without reviewing the target first.

## Claude Code

This repository includes a project-scoped `.mcp.json`. Claude Code will ask for
trust before using it. You can also add it explicitly:

```powershell
claude mcp add --transport http --scope project tileforge-actor-studio http://127.0.0.1:7331/mcp
claude mcp get tileforge-actor-studio
```

## Antigravity

Open **Manage MCP Servers → View raw config** and add this entry under
`mcpServers` in the shared Gemini/Antigravity MCP configuration:

```json
{
  "mcpServers": {
    "tileforge-actor-studio": {
      "serverUrl": "http://127.0.0.1:7331/mcp"
    }
  }
}
```

Antigravity 2.0 documents its shared configuration at
`~/.gemini/config/mcp_config.json`. Use the UI to locate the live file rather
than assuming a path on older installations.

## Local stdio fallback

For a client that cannot reach the localhost HTTP endpoint, launch:

```text
node --import tsx mcp/src/index.ts --transport stdio
```

Set the process working directory to the repository root. HTTP is preferred for
this project so every client observes the same gateway behavior.

## Available tools

- `get_studio_contract`
- `compile_actor_prompt`
- `create_sprite_session`
- `list_sprite_sessions`
- `get_sprite_session`

The gateway also exposes:

- resource: `studio://contracts/tileforge-actor-32-v1`
- prompt: `design_tileforge_actor`

No tool can approve final art.
