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
import {
  createExportCandidate,
  getExportCandidatePayload,
  listExportCandidates,
  validateExportCandidate,
} from "./exports.js";
import { createSession, getSession, listSessions, workspaceRoot } from "./storage.js";
import {
  createTurnaroundCandidate,
  getTurnaroundCandidatePayload,
  listTurnaroundCandidates,
  validateTurnaroundCandidate,
} from "./turnarounds.js";
import {
  createWalkCycleCandidate,
  getWalkCycleCandidatePayload,
  listWalkCycleCandidates,
  validateWalkCycleCandidate,
} from "./walk-cycles.js";
import {
  createWorldTestCandidate,
  getWorldTestCandidatePayload,
  listWorldTestCandidates,
  validateWorldTestCandidate,
} from "./world-tests.js";

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

const canonicalWalkCycleFramesSchema = z
  .array(canonicalPngBase64Schema)
  .length(TILEFORGE_ACTOR_CONTRACT.animation.framesPerDirection);

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

  server.registerTool(
    "create_walk_cycle_candidate",
    {
      title: "Create Walk Cycle candidate",
      description:
        "After the user explicitly accepts a Turnaround, atomically preserve four original walk frames per down/right/up/left direction at the contract timing. Frame 0 must exactly preserve each accepted Turnaround view. This records the user gate but cannot approve motion, final art, or publishing.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        sourceTurnaroundId: z.string().min(3).max(96),
        pngBase64: z
          .object({
            down: canonicalWalkCycleFramesSchema,
            right: canonicalWalkCycleFramesSchema,
            up: canonicalWalkCycleFramesSchema,
            left: canonicalWalkCycleFramesSchema,
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
    async ({ sessionId, sourceTurnaroundId, pngBase64, provenance }) =>
      textResult(
        await createWalkCycleCandidate(
          sessionId,
          sourceTurnaroundId,
          {
            down: pngBase64.down.map(decodeCanonicalBase64),
            right: pngBase64.right.map(decodeCanonicalBase64),
            up: pngBase64.up.map(decodeCanonicalBase64),
            left: pngBase64.left.map(decodeCanonicalBase64),
          },
          provenance,
          { root: storageRoot },
        ),
      ),
  );

  server.registerTool(
    "list_walk_cycle_candidates",
    {
      title: "List Walk Cycle candidates",
      description:
        "List immutable unreviewed Walk Cycle revisions for one studio session.",
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
        candidates: await listWalkCycleCandidates(sessionId, storageRoot),
      }),
  );

  server.registerTool(
    "get_walk_cycle_candidate",
    {
      title: "Get Walk Cycle candidate",
      description:
        "Read one immutable Walk Cycle document and all sixteen original frame PNGs. Motion and readability remain user judgments.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        walkCycleId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, walkCycleId }) => {
      const payload = await getWalkCycleCandidatePayload(
        sessionId,
        walkCycleId,
        storageRoot,
      );
      return textResult({
        candidate: payload.candidate,
        pngBase64: Object.fromEntries(
          Object.entries(payload.pngBytes).map(([direction, frames]) => [
            direction,
            frames.map((bytes) => Buffer.from(bytes).toString("base64")),
          ]),
        ),
      });
    },
  );

  server.registerTool(
    "validate_walk_cycle_candidate",
    {
      title: "Validate Walk Cycle candidate",
      description:
        "Recompute structural evidence for all sixteen immutable walk frames. This cannot decide whether motion or readability is acceptable; only the user can accept the Walk Cycle.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        walkCycleId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, walkCycleId }) =>
      textResult(
        await validateWalkCycleCandidate(
          sessionId,
          walkCycleId,
          storageRoot,
        ),
      ),
  );

  server.registerTool(
    "create_world_test_candidate",
    {
      title: "Create World Test candidate",
      description:
        "After the user explicitly accepts Walk Cycle motion/readability, atomically prepare immutable previews against the pinned TileForge reference pack. This records that transition but cannot approve final art, export, or publishing.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        sourceWalkCycleId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: false,
      },
    },
    async ({ sessionId, sourceWalkCycleId }) =>
      textResult(
        await createWorldTestCandidate(sessionId, sourceWalkCycleId, {
          root: storageRoot,
        }),
      ),
  );

  server.registerTool(
    "list_world_test_candidates",
    {
      title: "List World Test candidates",
      description:
        "List immutable, unreviewed World Test revisions for one studio session.",
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
        candidates: await listWorldTestCandidates(sessionId, storageRoot),
      }),
  );

  server.registerTool(
    "get_world_test_candidate",
    {
      title: "Get World Test candidate",
      description:
        "Read one immutable World Test document and its sixteen scene/theme previews. Final-art judgment remains user-owned.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        worldTestId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, worldTestId }) => {
      const payload = await getWorldTestCandidatePayload(
        sessionId,
        worldTestId,
        storageRoot,
      );
      return textResult({
        candidate: payload.candidate,
        previewPngBase64: Object.fromEntries(
          Object.entries(payload.previewPngBytes).map(([key, bytes]) => [
            key,
            Buffer.from(bytes).toString("base64"),
          ]),
        ),
      });
    },
  );

  server.registerTool(
    "validate_world_test_candidate",
    {
      title: "Validate World Test candidate",
      description:
        "Measure all sixteen immutable walk frames against sixteen pinned TileForge ground samples. This deterministic evidence cannot approve final art; only the user can do that.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        worldTestId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, worldTestId }) =>
      textResult(
        await validateWorldTestCandidate(
          sessionId,
          worldTestId,
          storageRoot,
        ),
      ),
  );

  server.registerTool(
    "create_export_candidate",
    {
      title: "Prepare draft Export",
      description:
        "Only after the user explicitly approves final art, atomically prepare an immutable local PNG sheet, metadata, and provenance package from that exact World Test. This records the user approval receipt but cannot approve or perform publishing.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        sourceWorldTestId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: false,
      },
    },
    async ({ sessionId, sourceWorldTestId }) =>
      textResult(
        await createExportCandidate(sessionId, sourceWorldTestId, {
          root: storageRoot,
        }),
      ),
  );

  server.registerTool(
    "list_export_candidates",
    {
      title: "List draft Exports",
      description:
        "List immutable local draft Export revisions. A listed package is not published and does not grant publishing approval.",
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
        candidates: await listExportCandidates(sessionId, storageRoot),
      }),
  );

  server.registerTool(
    "get_export_candidate",
    {
      title: "Get draft Export",
      description:
        "Read one immutable local draft Export, including its PNG sheet and parsed metadata and provenance. Publishing remains separately user-owned.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        exportId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, exportId }) => {
      const payload = await getExportCandidatePayload(
        sessionId,
        exportId,
        storageRoot,
      );
      return textResult({
        candidate: payload.candidate,
        spriteSheetPngBase64: Buffer.from(
          payload.spriteSheetPngBytes,
        ).toString("base64"),
        metadata: payload.metadata,
        provenance: payload.provenance,
      });
    },
  );

  server.registerTool(
    "validate_export_candidate",
    {
      title: "Validate draft Export",
      description:
        "Rehash the immutable package, reconstruct its 4 x 4 sheet from the approved source frames, and verify metadata, provenance, and the still-closed publishing boundary.",
      inputSchema: {
        sessionId: z.string().min(3).max(96),
        exportId: z.string().min(3).max(96),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ sessionId, exportId }) =>
      textResult(
        await validateExportCandidate(sessionId, exportId, storageRoot),
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
