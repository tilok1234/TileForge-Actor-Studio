import * as z from "zod/v4";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import { SESSION_ID_MAX_LENGTH } from "./session";

export const GENERATION_REQUEST_DOCUMENT_VERSION = 1;
export const GENERATION_REQUEST_ID_MAX_LENGTH = 96;
export const GENERATION_REQUEST_CANDIDATE_COUNT_DEFAULT = 3;

export const generationRequestIdSchema = z
  .string()
  .regex(
    /^[a-z0-9][a-z0-9-]{2,95}$/i,
    "Invalid generation request id.",
  )
  .max(GENERATION_REQUEST_ID_MAX_LENGTH);

export const conceptGenerationRequestSchema = z
  .object({
    schemaVersion: z.literal(GENERATION_REQUEST_DOCUMENT_VERSION),
    id: generationRequestIdSchema,
    revision: z.number().int().min(1),
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    stage: z.literal("concept"),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    createdAt: z.iso.datetime(),
    prompt: z.string().min(1).max(12_000),
    requestedCandidates: z.number().int().min(1).max(4),
    execution: z
      .object({
        mode: z.literal("connected-client-native-image-generation"),
        additionalPaidServices: z.literal("forbidden"),
        apiCredentials: z.literal("not-used"),
      })
      .strict(),
    output: z
      .object({
        artifact: z.literal("concept-candidate"),
        direction: z.literal("down"),
        width: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
        height: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
        mimeType: z.literal("image/png"),
        importTool: z.literal("import_concept_candidate"),
      })
      .strict(),
    authority: z
      .object({
        agentsMayGenerate: z.literal(true),
        agentsMayImport: z.literal(true),
        agentsMayApprove: z.literal(false),
        approvalOwner: z.literal("user"),
      })
      .strict(),
    lifecycle: z.literal("immutable-request"),
  })
  .strict();

export type ConceptGenerationRequest = z.infer<
  typeof conceptGenerationRequestSchema
>;

export function parseConceptGenerationRequest(
  value: unknown,
): ConceptGenerationRequest {
  return conceptGenerationRequestSchema.parse(value);
}

export function createConceptGenerationRequestId(
  revision: number,
  timestamp: string,
  idSuffix: string,
): string {
  const paddedRevision = revision.toString().padStart(4, "0");
  const timestampDigits = timestamp.replace(/\D/g, "").slice(0, 14);
  return generationRequestIdSchema.parse(
    `concept-gen-r${paddedRevision}-${timestampDigits}-${idSuffix}`,
  );
}
