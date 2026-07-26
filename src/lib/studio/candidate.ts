import * as z from "zod/v4";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import { SESSION_ID_MAX_LENGTH } from "./session";

export const CANDIDATE_DOCUMENT_VERSION = 1;
export const CANDIDATE_ID_MAX_LENGTH = 96;
export const CONCEPT_PNG_MAX_BYTES = 1_048_576;
export const candidateIdSchema = z
  .string()
  .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid candidate id.")
  .max(CANDIDATE_ID_MAX_LENGTH);
export const candidateSha256Schema = z.string().regex(/^[a-f0-9]{64}$/);

export const candidateProvenanceSchema = z
  .object({
    source: z.enum(["imported", "generated"]),
    originalFilename: z.string().trim().min(1).max(255).optional(),
    provider: z.string().trim().min(1).max(120).optional(),
    model: z.string().trim().min(1).max(120).optional(),
  })
  .strict()
  .superRefine((provenance, context) => {
    if (provenance.source === "generated" && !provenance.provider) {
      context.addIssue({
        code: "custom",
        path: ["provider"],
        message: "Generated candidates require provider provenance.",
      });
    }
  });

export type CandidateProvenance = z.infer<typeof candidateProvenanceSchema>;

const intakeValidationSchema = z
  .object({
    fileType: z.literal("pass"),
    dimensions: z.literal("pass"),
    alphaChannel: z.literal("pass"),
  })
  .strict();

export const conceptCandidateSchema = z
  .object({
    schemaVersion: z.literal(CANDIDATE_DOCUMENT_VERSION),
    id: candidateIdSchema,
    revision: z.number().int().min(1),
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    stage: z.literal("concept"),
    direction: z.literal("down"),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    sourceFile: z.literal("source.png"),
    mimeType: z.literal("image/png"),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1).max(CONCEPT_PNG_MAX_BYTES),
    width: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
    height: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
    createdAt: z.iso.datetime(),
    provenance: candidateProvenanceSchema,
    intakeValidation: intakeValidationSchema,
    reviewStatus: z.literal("unreviewed"),
  })
  .strict();

export type ConceptCandidate = z.infer<typeof conceptCandidateSchema>;

export function parseCandidateProvenance(value: unknown): CandidateProvenance {
  return candidateProvenanceSchema.parse(value);
}

export function parseConceptCandidate(value: unknown): ConceptCandidate {
  return conceptCandidateSchema.parse(value);
}

export function createConceptCandidateId(
  revision: number,
  timestamp: string,
  idSuffix: string,
): string {
  const paddedRevision = revision.toString().padStart(4, "0");
  const timestampDigits = timestamp.replace(/\D/g, "").slice(0, 14);
  const id = `concept-r${paddedRevision}-${timestampDigits}-${idSuffix}`;
  return conceptCandidateSchema.shape.id.parse(id);
}
