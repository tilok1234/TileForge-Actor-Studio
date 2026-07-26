import * as z from "zod/v4";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  TURNAROUND_DIRECTIONS,
} from "./turnaround";
import {
  WALK_CYCLE_FRAMES_PER_DIRECTION,
  walkCycleIdSchema,
} from "./walk-cycle";
import { validationReportSchema } from "./validation";

export const WALK_CYCLE_VALIDATION_REPORT_VERSION = 1;
export const WALK_CYCLE_VALIDATOR_ID =
  "tileforge-actor-32-walk-cycle-structural-v1";

const walkCycleFrameReportSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
    frameIndex: z
      .number()
      .int()
      .min(0)
      .max(WALK_CYCLE_FRAMES_PER_DIRECTION - 1),
    report: validationReportSchema,
  })
  .strict();

const summarySchema = z
  .object({
    pass: z.number().int().min(0),
    fail: z.number().int().min(0),
    notAssessed: z.number().int().min(0),
  })
  .strict();

export const walkCycleValidationReportSchema = z
  .object({
    schemaVersion: z.literal(WALK_CYCLE_VALIDATION_REPORT_VERSION),
    validatorId: z.literal(WALK_CYCLE_VALIDATOR_ID),
    walkCycleId: walkCycleIdSchema,
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    frames: z
      .array(walkCycleFrameReportSchema)
      .length(
        TURNAROUND_DIRECTIONS.length * WALK_CYCLE_FRAMES_PER_DIRECTION,
      ),
    summary: summarySchema,
    motionJudgment: z
      .object({
        status: z.literal("not_assessed"),
        authority: z.literal("user"),
        message: z.string().min(1),
      })
      .strict(),
  })
  .strict()
  .superRefine((validation, context) => {
    for (const [directionIndex, direction] of TURNAROUND_DIRECTIONS.entries()) {
      for (
        let frameIndex = 0;
        frameIndex < WALK_CYCLE_FRAMES_PER_DIRECTION;
        frameIndex += 1
      ) {
        const flatIndex =
          directionIndex * WALK_CYCLE_FRAMES_PER_DIRECTION + frameIndex;
        const frame = validation.frames[flatIndex];
        if (
          frame?.direction !== direction ||
          frame.frameIndex !== frameIndex
        ) {
          context.addIssue({
            code: "custom",
            path: ["frames", flatIndex],
            message:
              "Walk Cycle validation frames must use canonical direction and index order.",
          });
        }
        if (
          frame?.report.candidateId !== validation.walkCycleId ||
          frame.report.contractId !== validation.contractId
        ) {
          context.addIssue({
            code: "custom",
            path: ["frames", flatIndex, "report"],
            message:
              "Frame validation identity must match the Walk Cycle and contract.",
          });
        }
      }
    }

    const counted = validation.frames.reduce(
      (summary, frame) => ({
        pass: summary.pass + frame.report.summary.pass,
        fail: summary.fail + frame.report.summary.fail,
        notAssessed:
          summary.notAssessed + frame.report.summary.notAssessed,
      }),
      { pass: 0, fail: 0, notAssessed: 0 },
    );
    if (
      counted.pass !== validation.summary.pass ||
      counted.fail !== validation.summary.fail ||
      counted.notAssessed !== validation.summary.notAssessed
    ) {
      context.addIssue({
        code: "custom",
        path: ["summary"],
        message:
          "Walk Cycle validation summary does not match frame reports.",
      });
    }
  });

export type WalkCycleValidationReport = z.infer<
  typeof walkCycleValidationReportSchema
>;

export function parseWalkCycleValidationReport(
  value: unknown,
): WalkCycleValidationReport {
  return walkCycleValidationReportSchema.parse(value);
}
