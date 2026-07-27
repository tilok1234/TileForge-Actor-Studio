import * as z from "zod/v4";
import { candidateSha256Schema } from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  WORLD_TEST_PREVIEW_HEIGHT,
  WORLD_TEST_PREVIEW_WIDTH,
  WORLD_TEST_REFERENCE_PACK_ID,
  WORLD_TEST_SCENES,
  WORLD_TEST_THEMES,
} from "./reference-pack";
import { SESSION_ID_MAX_LENGTH } from "./session";
import { TURNAROUND_DIRECTIONS } from "./turnaround";
import {
  WALK_CYCLE_FRAMES_PER_DIRECTION,
  walkCycleIdSchema,
} from "./walk-cycle";

export const WORLD_TEST_DOCUMENT_VERSION = 1;
export const WORLD_TEST_ID_MAX_LENGTH = 96;

export const worldTestIdSchema = z
  .string()
  .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid World Test id.")
  .max(WORLD_TEST_ID_MAX_LENGTH);

const acceptedFrameSourceSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
    frameIndex: z
      .number()
      .int()
      .min(0)
      .max(WALK_CYCLE_FRAMES_PER_DIRECTION - 1),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
  })
  .strict();

const sourceWalkCycleSchema = z
  .object({
    walkCycleId: walkCycleIdSchema,
    frameSources: z
      .array(acceptedFrameSourceSchema)
      .length(
        TURNAROUND_DIRECTIONS.length * WALK_CYCLE_FRAMES_PER_DIRECTION,
      ),
    acceptedBy: z.literal("user"),
    acceptedAt: z.iso.datetime(),
  })
  .strict();

const referencePackReceiptSchema = z
  .object({
    id: z.literal(WORLD_TEST_REFERENCE_PACK_ID),
    version: z.literal(1),
    manifestSha256: candidateSha256Schema,
    checkoutCommit: z.string().regex(/^[a-f0-9]{40}$/),
    generatedEngineCommit: z.string().regex(/^[a-f0-9]{7,40}$/),
  })
  .strict();

const previewSourceSchema = z
  .object({
    scene: z.enum(WORLD_TEST_SCENES),
    theme: z.enum(WORLD_TEST_THEMES),
    sourceFile: z.string().regex(/^[a-z0-9-]+\.png$/),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
    width: z.literal(WORLD_TEST_PREVIEW_WIDTH),
    height: z.literal(WORLD_TEST_PREVIEW_HEIGHT),
    referenceSourceSha256: candidateSha256Schema,
  })
  .strict();

const finalArtJudgmentSchema = z
  .object({
    status: z.literal("not_assessed"),
    authority: z.literal("user"),
    message: z.string().min(1),
  })
  .strict();

export const worldTestCandidateSchema = z
  .object({
    schemaVersion: z.literal(WORLD_TEST_DOCUMENT_VERSION),
    id: worldTestIdSchema,
    revision: z.number().int().min(1),
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    stage: z.literal("world-test"),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    sourceWalkCycle: sourceWalkCycleSchema,
    referencePack: referencePackReceiptSchema,
    previews: z
      .array(previewSourceSchema)
      .length(WORLD_TEST_SCENES.length * WORLD_TEST_THEMES.length),
    createdAt: z.iso.datetime(),
    preparation: z
      .object({
        method: z.literal("local-deterministic-compositor-v1"),
        additionalAiCost: z.literal(false),
      })
      .strict(),
    reviewStatus: z.literal("unreviewed"),
    finalArtJudgment: finalArtJudgmentSchema,
  })
  .strict()
  .superRefine((candidate, context) => {
    for (const [directionIndex, direction] of TURNAROUND_DIRECTIONS.entries()) {
      for (
        let frameIndex = 0;
        frameIndex < WALK_CYCLE_FRAMES_PER_DIRECTION;
        frameIndex += 1
      ) {
        const index =
          directionIndex * WALK_CYCLE_FRAMES_PER_DIRECTION + frameIndex;
        const frame = candidate.sourceWalkCycle.frameSources[index];
        if (
          frame?.direction !== direction ||
          frame.frameIndex !== frameIndex
        ) {
          context.addIssue({
            code: "custom",
            path: ["sourceWalkCycle", "frameSources", index],
            message:
              "Accepted Walk Cycle frames must use canonical direction and frame order.",
          });
        }
      }
    }
    for (const [sceneIndex, scene] of WORLD_TEST_SCENES.entries()) {
      for (const [themeIndex, theme] of WORLD_TEST_THEMES.entries()) {
        const index = sceneIndex * WORLD_TEST_THEMES.length + themeIndex;
        const preview = candidate.previews[index];
        if (
          preview?.scene !== scene ||
          preview.theme !== theme ||
          preview.sourceFile !== `${scene}-${theme}.png`
        ) {
          context.addIssue({
            code: "custom",
            path: ["previews", index],
            message:
              "World Test previews must use canonical scene, theme, and filename order.",
          });
        }
      }
    }
  });

export type WorldTestCandidate = z.infer<typeof worldTestCandidateSchema>;
export type WorldTestPreviewSource = z.infer<typeof previewSourceSchema>;

export function parseWorldTestCandidate(value: unknown): WorldTestCandidate {
  return worldTestCandidateSchema.parse(value);
}
export function createWorldTestCandidateId(
  revision: number,
  timestamp: string,
  idSuffix: string,
): string {
  const paddedRevision = revision.toString().padStart(4, "0");
  const timestampDigits = timestamp.replace(/\D/g, "").slice(0, 14);
  return worldTestIdSchema.parse(
    `world-test-r${paddedRevision}-${timestampDigits}-${idSuffix}`,
  );
}
