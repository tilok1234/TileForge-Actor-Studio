import * as z from "zod/v4";
import {
  candidateIdSchema,
  candidateProvenanceSchema,
  candidateSha256Schema,
} from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import { SESSION_ID_MAX_LENGTH } from "./session";

export const TURNAROUND_DOCUMENT_VERSION = 1;
export const TURNAROUND_ID_MAX_LENGTH = 96;
export const TURNAROUND_DIRECTIONS = ["down", "right", "up", "left"] as const;

export const turnaroundIdSchema = z
  .string()
  .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid turnaround id.")
  .max(TURNAROUND_ID_MAX_LENGTH);

const sourceSelectionSchema = z
  .object({
    candidateId: candidateIdSchema,
    candidateSha256: candidateSha256Schema,
    selectedBy: z.literal("user"),
    selectedAt: z.iso.datetime(),
  })
  .strict();

const directionSourceSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
    sourceFile: z.enum([
      "down.png",
      "right.png",
      "up.png",
      "left.png",
    ]),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
    width: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
    height: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
  })
  .strict();

const identityJudgmentSchema = z
  .object({
    status: z.literal("not_assessed"),
    authority: z.literal("user"),
    message: z.string().min(1),
  })
  .strict();

export const turnaroundCandidateSchema = z
  .object({
    schemaVersion: z.literal(TURNAROUND_DOCUMENT_VERSION),
    id: turnaroundIdSchema,
    revision: z.number().int().min(1),
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    stage: z.literal("turnaround"),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    sourceSelection: sourceSelectionSchema,
    directions: z.array(directionSourceSchema).length(TURNAROUND_DIRECTIONS.length),
    createdAt: z.iso.datetime(),
    provenance: candidateProvenanceSchema,
    reviewStatus: z.literal("unreviewed"),
    identityJudgment: identityJudgmentSchema,
  })
  .strict()
  .superRefine((candidate, context) => {
    for (const [index, direction] of TURNAROUND_DIRECTIONS.entries()) {
      const source = candidate.directions[index];
      if (
        source?.direction !== direction ||
        source.sourceFile !== `${direction}.png`
      ) {
        context.addIssue({
          code: "custom",
          path: ["directions", index],
          message: "Turnaround directions must use canonical order and filenames.",
        });
      }
    }
    const down = candidate.directions[0];
    if (down?.sha256 !== candidate.sourceSelection.candidateSha256) {
      context.addIssue({
        code: "custom",
        path: ["directions", 0, "sha256"],
        message: "Down view must preserve the selected Concept bytes.",
      });
    }
  });

export type TurnaroundDirection = (typeof TURNAROUND_DIRECTIONS)[number];
export type TurnaroundCandidate = z.infer<typeof turnaroundCandidateSchema>;
export type TurnaroundDirectionSource = z.infer<typeof directionSourceSchema>;

export function parseTurnaroundCandidate(value: unknown): TurnaroundCandidate {
  return turnaroundCandidateSchema.parse(value);
}

export function createTurnaroundCandidateId(
  revision: number,
  timestamp: string,
  idSuffix: string,
): string {
  const paddedRevision = revision.toString().padStart(4, "0");
  const timestampDigits = timestamp.replace(/\D/g, "").slice(0, 14);
  return turnaroundIdSchema.parse(
    `turnaround-r${paddedRevision}-${timestampDigits}-${idSuffix}`,
  );
}
