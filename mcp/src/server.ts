import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import * as z from "zod/v4";
import { actorBriefInputSchema } from "../../src/lib/studio/brief.js";
import {
  candidateProvenanceSchema,
  CONCEPT_PNG_MAX_BYTES,
} from "../../src/lib/studio/candidate.js";
import { TILEFORGE_ACTOR_CONTRACT } from "../../src/lib/studio/contract.js";
import { compileActorPrompt } from "../../src/lib/studio/prompt.js";
import { validateConceptCandidatePng } from "../../src/lib/studio/validate-candidate.js";
import {
  createConceptCandidate,
  getConceptCandidatePayload,
  listConceptCandidates,
} from "./candidates.js";
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

function decodeCanonicalBase64(value: string): Uint8Array {
  const decoded = Buffer.from(value, "base64");
  if (decoded.toString("base64") !== value) {
    throw new Error("pngBase64 must use canonical base64 encoding.");
  }
  return decoded;
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

  server.registerTool(
    "import_concept_candidate",
    {
      title: "Import concept candidate",
      description:
        "Atomically preserve one original 32 x 32 down-facing PNG with provenance. This creates an unreviewed candidate and cannot approve art.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        pngBase64: z
          .string()
          .min(1)
          .max(Math.ceil((CONCEPT_PNG_MAX_BYTES * 4) / 3) + 4)
          .regex(/^[A-Za-z0-9+/]+={0,2}$/),
        provenance: candidateProvenanceSchema,
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: false,
      },
    },
    async ({ sessionId, pngBase64, provenance }) =>
      textResult(
        await createConceptCandidate(
          sessionId,
          decodeCanonicalBase64(pngBase64),
          provenance,
          { root: storageRoot },
        ),
      ),
  );

  server.registerTool(
    "list_concept_candidates",
    {
      title: "List concept candidates",
      description:
        "List immutable unreviewed Concept candidates for one studio session, newest revision first.",
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
    async ({ sessionId }) =>
      textResult({
        candidates: await listConceptCandidates(sessionId, storageRoot),
      }),
  );

  server.registerTool(
    "get_concept_candidate",
    {
      title: "Get concept candidate",
      description:
        "Read one immutable Concept candidate and its original PNG bytes. Structural intake is evidence, not visual approval.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        candidateId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, candidateId }) => {
      const payload = await getConceptCandidatePayload(
        sessionId,
        candidateId,
        storageRoot,
      );
      return textResult({
        candidate: payload.candidate,
        pngBase64: Buffer.from(payload.pngBytes).toString("base64"),
      });
    },
  );

  server.registerTool(
    "validate_concept_candidate",
    {
      title: "Validate concept candidate",
      description:
        "Measure the local structural contract against one immutable candidate. Pass or Fail evidence is not visual approval; only the user can accept final art.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        candidateId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, candidateId }) => {
      const payload = await getConceptCandidatePayload(
        sessionId,
        candidateId,
        storageRoot,
      );
      return textResult(
        validateConceptCandidatePng(payload.candidate, payload.pngBytes),
      );
    },
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
