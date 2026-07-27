import * as z from "zod/v4";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  WORLD_TEST_SCENES,
  WORLD_TEST_THEMES,
} from "./reference-pack";
import { TURNAROUND_DIRECTIONS } from "./turnaround";
import { WALK_CYCLE_FRAMES_PER_DIRECTION } from "./walk-cycle";
import { worldTestIdSchema } from "./world-test";

export const WORLD_TEST_VALIDATION_REPORT_VERSION = 1;
export const WORLD_TEST_VALIDATOR_ID =
  "tileforge-actor-32-world-test-ground-luma-v1";

const measurementSchema = z
  .object({
    scene: z.enum(WORLD_TEST_SCENES),
    theme: z.enum(WORLD_TEST_THEMES),
    direction: z.enum(TURNAROUND_DIRECTIONS),
    frameIndex: z
      .number()
      .int()
      .min(0)
      .max(WALK_CYCLE_FRAMES_PER_DIRECTION - 1),
    actorMeanLuma: z.number().int().min(0).max(255),
    groundMeanLuma: z.number().int().min(0).max(255),
    distance: z.number().int().min(0).max(255),
    minimum: z.literal(
      TILEFORGE_ACTOR_CONTRACT.art.minimumGroundLumaDistance,
    ),
    status: z.enum(["pass", "fail"]),
  })
  .strict();

const summarySchema = z
  .object({
    pass: z.number().int().min(0),
    fail: z.number().int().min(0),
    notAssessed: z.literal(0),
  })
  .strict();

export const worldTestValidationReportSchema = z
  .object({
    schemaVersion: z.literal(WORLD_TEST_VALIDATION_REPORT_VERSION),
    validatorId: z.literal(WORLD_TEST_VALIDATOR_ID),
    worldTestId: worldTestIdSchema,
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    measurements: z
      .array(measurementSchema)
      .length(
        WORLD_TEST_SCENES.length *
          WORLD_TEST_THEMES.length *
          TURNAROUND_DIRECTIONS.length *
          WALK_CYCLE_FRAMES_PER_DIRECTION,
      ),
    summary: summarySchema,
    finalArtJudgment: z
      .object({
        status: z.literal("not_assessed"),
        authority: z.literal("user"),
        message: z.string().min(1),
      })
      .strict(),
  })
  .strict()
  .superRefine((report, context) => {
    let index = 0;
    for (const scene of WORLD_TEST_SCENES) {
      for (const theme of WORLD_TEST_THEMES) {
        for (const direction of TURNAROUND_DIRECTIONS) {
          for (
            let frameIndex = 0;
            frameIndex < WALK_CYCLE_FRAMES_PER_DIRECTION;
            frameIndex += 1
          ) {
            const measurement = report.measurements[index];
            if (
              measurement?.scene !== scene ||
              measurement.theme !== theme ||
              measurement.direction !== direction ||
              measurement.frameIndex !== frameIndex
            ) {
              context.addIssue({
                code: "custom",
                path: ["measurements", index],
                message:
                  "World Test measurements must use canonical reference and frame order.",
              });
            } else if (
              measurement.distance !==
                Math.abs(
                  measurement.actorMeanLuma - measurement.groundMeanLuma,
                ) ||
              (measurement.status === "pass") !==
                (measurement.distance >= measurement.minimum)
            ) {
              context.addIssue({
                code: "custom",
                path: ["measurements", index],
                message:
                  "World Test luma distance or status is inconsistent.",
              });
            }
            index += 1;
          }
        }
      }
    }
    const counted = report.measurements.reduce(
      (summary, measurement) => {
        summary[measurement.status] += 1;
        return summary;
      },
      { pass: 0, fail: 0 },
    );
    if (
      counted.pass !== report.summary.pass ||
      counted.fail !== report.summary.fail
    ) {
      context.addIssue({
        code: "custom",
        path: ["summary"],
        message: "World Test validation summary does not match measurements.",
      });
    }
  });

export type WorldTestValidationReport = z.infer<
  typeof worldTestValidationReportSchema
>;

export function parseWorldTestValidationReport(
  value: unknown,
): WorldTestValidationReport {
  return worldTestValidationReportSchema.parse(value);
}
