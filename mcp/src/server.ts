import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import * as z from "zod/v4";
import { actorBriefInputSchema } from "../../src/lib/studio/brief.js";
import { TILEFORGE_ACTOR_CONTRACT } from "../../src/lib/studio/contract.js";
import { compileActorPrompt } from "../../src/lib/studio/prompt.js";
import { createSession, getSession, listSessions, workspaceRoot } from "./storage.js";

function textResult(value: unknown) {
  return {
    content: [
      {
        type: "text" as const,
        text: JSON.stringify(value, null, 2),
      },
    ],
  };
}

interface ActorStudioServerOptions {
  workspaceRoot?: string;
}

export function createActorStudioServer(
  options: ActorStudioServerOptions = {},
): McpServer {
  const storageRoot = options.workspaceRoot ?? workspaceRoot;
  const server = new McpServer({
    name: "tileforge-actor-studio",
    version: "0.1.0",
  });

  server.registerResource(
    "tileforge-actor-contract",
    "studio://contracts/tileforge-actor-32-v1",
    {
      title: "TileForge 32px actor contract",
      description: "The immutable world, frame, art, animation, and approval boundaries.",
      mimeType: "application/json",
    },
    async (uri) => ({
      contents: [
        {
          uri: uri.href,
          mimeType: "application/json",
          text: JSON.stringify(TILEFORGE_ACTOR_CONTRACT, null, 2),
        },
      ],
    }),
  );

  server.registerTool(
    "get_studio_contract",
    {
      title: "Get studio contract",
      description:
        "Read the locked TileForge actor contract before designing or validating an actor.",
      inputSchema: {},
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async () => textResult(TILEFORGE_ACTOR_CONTRACT),
  );

  server.registerTool(
    "compile_actor_prompt",
    {
      title: "Compile actor prompt",
      description:
        "Combine a creative actor brief with the locked TileForge constraints. This does not generate or modify art.",
      inputSchema: actorBriefInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async (brief) =>
      textResult({
        contractId: TILEFORGE_ACTOR_CONTRACT.id,
        prompt: compileActorPrompt(brief),
      }),
  );

  server.registerTool(
    "create_sprite_session",
    {
      title: "Create sprite session",
      description:
        "Create a versioned local workspace for one mob or NPC. Final art approval remains unavailable to agents.",
      inputSchema: actorBriefInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: false,
      },
    },
    async (brief) => textResult(await createSession(brief, { root: storageRoot })),
  );

  server.registerTool(
    "list_sprite_sessions",
    {
      title: "List sprite sessions",
      description: "List local Actor Studio sessions, newest first.",
      inputSchema: {},
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async () =>
      textResult({
        workspaceRoot: storageRoot,
        sessions: await listSessions(storageRoot),
      }),
  );

  server.registerTool(
    "get_sprite_session",
    {
      title: "Get sprite session",
      description: "Read one Actor Studio session by its stable id.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId }) => textResult(await getSession(sessionId, storageRoot)),
  );

  server.registerPrompt(
    "design_tileforge_actor",
    {
      title: "Design TileForge actor",
      description:
        "Start an actor-design conversation using the same locked contract as the desktop studio.",
      argsSchema: actorBriefInputSchema,
    },
    async (brief) => ({
      messages: [
        {
          role: "user",
          content: {
            type: "text",
            text: compileActorPrompt(brief),
          },
        },
      ],
    }),
  );

  return server;
}
