import * as z from "zod/v4";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  TURNAROUND_DIRECTIONS,
  turnaroundIdSchema,
} from "./turnaround";
import { validationReportSchema } from "./validation";

export const TURNAROUND_VALIDATION_REPORT_VERSION = 1;
export const TURNAROUND_VALIDATOR_ID =
  "tileforge-actor-32-turnaround-structural-v1";

const turnaroundDirectionReportSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
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

export const turnaroundValidationReportSchema = z
  .object({
    schemaVersion: z.literal(TURNAROUND_VALIDATION_REPORT_VERSION),
    validatorId: z.literal(TURNAROUND_VALIDATOR_ID),
    turnaroundId: turnaroundIdSchema,
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    directions: z
      .array(turnaroundDirectionReportSchema)
      .length(TURNAROUND_DIRECTIONS.length),
    summary: summarySchema,
    identityJudgment: z
      .object({
        status: z.literal("not_assessed"),
        authority: z.literal("user"),
        message: z.string().min(1),
      })
      .strict(),
  })
  .strict()
  .superRefine((validation, context) => {
    for (const [index, direction] of TURNAROUND_DIRECTIONS.entries()) {
      const directionReport = validation.directions[index];
      if (directionReport?.direction !== direction) {
        context.addIssue({
          code: "custom",
          path: ["directions", index, "direction"],
          message: "Turnaround validation directions must use canonical order.",
        });
      }
      if (directionReport?.report.candidateId !== validation.turnaroundId) {
        context.addIssue({
          code: "custom",
          path: ["directions", index, "report", "candidateId"],
          message: "Direction report identity must match the Turnaround.",
        });
      }
      if (directionReport?.report.contractId !== validation.contractId) {
        context.addIssue({
          code: "custom",
          path: ["directions", index, "report", "contractId"],
          message: "Direction report contract must match the Turnaround.",
        });
      }
    }
    const counted = validation.directions.reduce(
      (summary, direction) => ({
        pass: summary.pass + direction.report.summary.pass,
        fail: summary.fail + direction.report.summary.fail,
        notAssessed:
          summary.notAssessed + direction.report.summary.notAssessed,
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
        message: "Turnaround validation summary does not match direction reports.",
      });
    }
  });

export type TurnaroundValidationReport = z.infer<
  typeof turnaroundValidationReportSchema
>;

export function parseTurnaroundValidationReport(
  value: unknown,
): TurnaroundValidationReport {
  return turnaroundValidationReportSchema.parse(value);
}
