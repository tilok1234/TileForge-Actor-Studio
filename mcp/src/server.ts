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
import {
  createTurnaroundCandidate,
  getTurnaroundCandidatePayload,
  listTurnaroundCandidates,
  validateTurnaroundCandidate,
} from "./turnarounds.js";

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

const canonicalPngBase64Schema = z
  .string()
  .min(1)
  .max(Math.ceil((CONCEPT_PNG_MAX_BYTES * 4) / 3) + 4)
  .regex(/^[A-Za-z0-9+/]+={0,2}$/);

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
        pngBase64: canonicalPngBase64Schema,
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

  server.registerTool(
    "create_turnaround_candidate",
    {
      title: "Create turnaround candidate",
      description:
        "After the user explicitly selects a Concept, atomically preserve exact down/right/up/left PNG views as one immutable unreviewed Turnaround revision. This records user selection but cannot accept identity consistency or approve final art.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        sourceConceptId: z.string().min(3).max(96),
        pngBase64: z
          .object({
            down: canonicalPngBase64Schema,
            right: canonicalPngBase64Schema,
            up: canonicalPngBase64Schema,
            left: canonicalPngBase64Schema,
          })
          .strict(),
        provenance: candidateProvenanceSchema,
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: false,
      },
    },
    async ({ sessionId, sourceConceptId, pngBase64, provenance }) =>
      textResult(
        await createTurnaroundCandidate(
          sessionId,
          sourceConceptId,
          {
            down: decodeCanonicalBase64(pngBase64.down),
            right: decodeCanonicalBase64(pngBase64.right),
            up: decodeCanonicalBase64(pngBase64.up),
            left: decodeCanonicalBase64(pngBase64.left),
          },
          provenance,
          { root: storageRoot },
        ),
      ),
  );

  server.registerTool(
    "list_turnaround_candidates",
    {
      title: "List turnaround candidates",
      description:
        "List immutable unreviewed Turnaround revisions for one studio session.",
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
        candidates: await listTurnaroundCandidates(sessionId, storageRoot),
      }),
  );

  server.registerTool(
    "get_turnaround_candidate",
    {
      title: "Get turnaround candidate",
      description:
        "Read one immutable Turnaround document and all four original direction PNGs. Identity consistency remains a user judgment.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        turnaroundId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, turnaroundId }) => {
      const payload = await getTurnaroundCandidatePayload(
        sessionId,
        turnaroundId,
        storageRoot,
      );
      return textResult({
        candidate: payload.candidate,
        pngBase64: Object.fromEntries(
          Object.entries(payload.pngBytes).map(([direction, bytes]) => [
            direction,
            Buffer.from(bytes).toString("base64"),
          ]),
        ),
      });
    },
  );

  server.registerTool(
    "validate_turnaround_candidate",
    {
      title: "Validate turnaround candidate",
      description:
        "Recompute structural evidence for all four immutable direction PNGs. This cannot decide whether identity is visually consistent; only the user can accept the Turnaround.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        turnaroundId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, turnaroundId }) =>
      textResult(
        await validateTurnaroundCandidate(
          sessionId,
          turnaroundId,
          storageRoot,
        ),
      ),
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
