import * as z from "zod/v4";
import { conceptCandidateSchema } from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";

export const VALIDATION_REPORT_VERSION = 1;
export const STRUCTURAL_VALIDATOR_ID = "tileforge-actor-32-structural-v1";
export const VALIDATION_RULE_IDS = [
  "canvas_dimensions",
  "hard_alpha",
  "actor_height",
  "foot_anchor",
  "palette_max_colors",
  "ground_luma_separation",
  "frame_edge_clipping",
] as const;

export const validationStatusSchema = z.enum([
  "pass",
  "fail",
  "not_assessed",
]);

export const validationRuleResultSchema = z
  .object({
    id: z.enum(VALIDATION_RULE_IDS),
    status: validationStatusSchema,
    expected: z.string().min(1),
    observed: z.string().min(1).nullable(),
    message: z.string().min(1),
  })
  .strict();

const validationSummarySchema = z
  .object({
    pass: z.number().int().min(0),
    fail: z.number().int().min(0),
    notAssessed: z.number().int().min(0),
  })
  .strict();

const visualJudgmentSchema = z
  .object({
    status: z.literal("not_assessed"),
    authority: z.literal("user"),
    message: z.string().min(1),
  })
  .strict();

export const validationReportSchema = z
  .object({
    schemaVersion: z.literal(VALIDATION_REPORT_VERSION),
    validatorId: z.literal(STRUCTURAL_VALIDATOR_ID),
    candidateId: conceptCandidateSchema.shape.id,
    candidateSha256: conceptCandidateSchema.shape.sha256,
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    results: z
      .array(validationRuleResultSchema)
      .length(VALIDATION_RULE_IDS.length),
    summary: validationSummarySchema,
    visualJudgment: visualJudgmentSchema,
  })
  .strict()
  .superRefine((report, context) => {
    for (const [index, ruleId] of VALIDATION_RULE_IDS.entries()) {
      if (report.results[index]?.id !== ruleId) {
        context.addIssue({
          code: "custom",
          path: ["results", index, "id"],
          message: "Validation rules must use the canonical order.",
        });
      }
    }

    const counted = report.results.reduce(
      (summary, result) => {
        if (result.status === "not_assessed") {
          summary.notAssessed += 1;
        } else {
          summary[result.status] += 1;
        }
        return summary;
      },
      { pass: 0, fail: 0, notAssessed: 0 },
    );
    if (
      counted.pass !== report.summary.pass ||
      counted.fail !== report.summary.fail ||
      counted.notAssessed !== report.summary.notAssessed
    ) {
      context.addIssue({
        code: "custom",
        path: ["summary"],
        message: "Validation summary does not match its rule results.",
      });
    }
  });

export type ValidationStatus = z.infer<typeof validationStatusSchema>;
export type ValidationRuleResult = z.infer<
  typeof validationRuleResultSchema
>;
export type ValidationReport = z.infer<typeof validationReportSchema>;

export function parseValidationReport(value: unknown): ValidationReport {
  return validationReportSchema.parse(value);
}
