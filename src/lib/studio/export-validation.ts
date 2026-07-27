import * as z from "zod/v4";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  exportIdSchema,
  publishingBoundarySchema,
} from "./export";

export const EXPORT_VALIDATION_REPORT_VERSION = 1;
export const EXPORT_VALIDATOR_ID =
  "tileforge-actor-32-export-package-v1";
export const EXPORT_VALIDATION_CHECKS = [
  "approved_world_test",
  "source_walk_cycle",
  "sprite_sheet_identity",
  "sprite_sheet_pixels",
  "metadata_identity",
  "provenance_identity",
  "publishing_boundary",
] as const;

const checkSchema = z
  .object({
    id: z.enum(EXPORT_VALIDATION_CHECKS),
    status: z.literal("pass"),
    message: z.string().min(1),
  })
  .strict();

export const exportValidationReportSchema = z
  .object({
    schemaVersion: z.literal(EXPORT_VALIDATION_REPORT_VERSION),
    validatorId: z.literal(EXPORT_VALIDATOR_ID),
    exportId: exportIdSchema,
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    checks: z.array(checkSchema).length(EXPORT_VALIDATION_CHECKS.length),
    summary: z
      .object({
        pass: z.literal(EXPORT_VALIDATION_CHECKS.length),
        fail: z.literal(0),
        notAssessed: z.literal(0),
      })
      .strict(),
    publishing: publishingBoundarySchema,
  })
  .strict()
  .superRefine((report, context) => {
    for (const [index, id] of EXPORT_VALIDATION_CHECKS.entries()) {
      if (report.checks[index]?.id !== id) {
        context.addIssue({
          code: "custom",
          path: ["checks", index, "id"],
          message: "Export checks must use canonical order.",
        });
      }
    }
  });

export type ExportValidationReport = z.infer<
  typeof exportValidationReportSchema
>;

export function parseExportValidationReport(
  value: unknown,
): ExportValidationReport {
  return exportValidationReportSchema.parse(value);
}
