import * as z from "zod/v4";
import {
  candidateProvenanceSchema,
  candidateSha256Schema,
} from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import { SESSION_ID_MAX_LENGTH } from "./session";
import {
  TURNAROUND_DIRECTIONS,
  turnaroundIdSchema,
} from "./turnaround";

export const WALK_CYCLE_DOCUMENT_VERSION = 1;
export const WALK_CYCLE_ID_MAX_LENGTH = 96;
export const WALK_CYCLE_FRAMES_PER_DIRECTION =
  TILEFORGE_ACTOR_CONTRACT.animation.framesPerDirection;
export const WALK_CYCLE_FRAME_DURATION_MS =
  TILEFORGE_ACTOR_CONTRACT.animation.frameDurationMs;

export const walkCycleIdSchema = z
  .string()
  .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid Walk Cycle id.")
  .max(WALK_CYCLE_ID_MAX_LENGTH);

const acceptedDirectionSourceSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
  })
  .strict();

const sourceTurnaroundSchema = z
  .object({
    turnaroundId: turnaroundIdSchema,
    directionSources: z
      .array(acceptedDirectionSourceSchema)
      .length(TURNAROUND_DIRECTIONS.length),
    acceptedBy: z.literal("user"),
    acceptedAt: z.iso.datetime(),
  })
  .strict();

const walkCycleFrameSourceSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
    frameIndex: z.number().int().min(0).max(WALK_CYCLE_FRAMES_PER_DIRECTION - 1),
    sourceFile: z.string().min(1),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
    width: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
    height: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
  })
  .strict();

const motionJudgmentSchema = z
  .object({
    status: z.literal("not_assessed"),
    authority: z.literal("user"),
    message: z.string().min(1),
  })
  .strict();

export const walkCycleCandidateSchema = z
  .object({
    schemaVersion: z.literal(WALK_CYCLE_DOCUMENT_VERSION),
    id: walkCycleIdSchema,
    revision: z.number().int().min(1),
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    stage: z.literal("animate"),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    sourceTurnaround: sourceTurnaroundSchema,
    clip: z.literal("walk"),
    framesPerDirection: z.literal(WALK_CYCLE_FRAMES_PER_DIRECTION),
    frameDurationMs: z.literal(WALK_CYCLE_FRAME_DURATION_MS),
    frames: z
      .array(walkCycleFrameSourceSchema)
      .length(
        TURNAROUND_DIRECTIONS.length * WALK_CYCLE_FRAMES_PER_DIRECTION,
      ),
    createdAt: z.iso.datetime(),
    provenance: candidateProvenanceSchema,
    reviewStatus: z.literal("unreviewed"),
    motionJudgment: motionJudgmentSchema,
  })
  .strict()
  .superRefine((candidate, context) => {
    for (const [directionIndex, direction] of TURNAROUND_DIRECTIONS.entries()) {
      const acceptedSource =
        candidate.sourceTurnaround.directionSources[directionIndex];
      if (acceptedSource?.direction !== direction) {
        context.addIssue({
          code: "custom",
          path: [
            "sourceTurnaround",
            "directionSources",
            directionIndex,
            "direction",
          ],
          message:
            "Accepted Turnaround directions must use canonical contract order.",
        });
      }

      for (
        let frameIndex = 0;
        frameIndex < WALK_CYCLE_FRAMES_PER_DIRECTION;
        frameIndex += 1
      ) {
        const flatIndex =
          directionIndex * WALK_CYCLE_FRAMES_PER_DIRECTION + frameIndex;
        const frame = candidate.frames[flatIndex];
        if (
          frame?.direction !== direction ||
          frame.frameIndex !== frameIndex ||
          frame.sourceFile !== `${direction}-${frameIndex}.png`
        ) {
          context.addIssue({
            code: "custom",
            path: ["frames", flatIndex],
            message:
              "Walk Cycle frames must use canonical direction, index, and filename order.",
          });
        }
        if (
          frameIndex === 0 &&
          acceptedSource &&
          (frame?.sha256 !== acceptedSource.sha256 ||
            frame.byteLength !== acceptedSource.byteLength)
        ) {
          context.addIssue({
            code: "custom",
            path: ["frames", flatIndex, "sha256"],
            message:
              "Frame 0 must preserve the accepted Turnaround direction bytes.",
          });
        }
      }
    }
  });

export type WalkCycleCandidate = z.infer<typeof walkCycleCandidateSchema>;
export type WalkCycleFrameSource = z.infer<
  typeof walkCycleFrameSourceSchema
>;

export function parseWalkCycleCandidate(value: unknown): WalkCycleCandidate {
  return walkCycleCandidateSchema.parse(value);
}

export function createWalkCycleCandidateId(
  revision: number,
  timestamp: string,
  idSuffix: string,
): string {
  const paddedRevision = revision.toString().padStart(4, "0");
  const timestampDigits = timestamp.replace(/\D/g, "").slice(0, 14);
  return walkCycleIdSchema.parse(
    `walk-cycle-r${paddedRevision}-${timestampDigits}-${idSuffix}`,
  );
}
