import * as z from "zod/v4";
import { actorBriefSchema, parseActorBrief } from "./brief";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import type { ActorBrief, StudioSession } from "./types";

export const SESSION_ID_MAX_LENGTH = 96;
export const SESSION_SLUG_MAX_LENGTH = 64;

const studioStageSchema = z.enum([
  "brief",
  "concept",
  "turnaround",
  "animate",
  "world-test",
  "export",
]);

export const studioSessionSchema = z.object({
  id: z
    .string()
    .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
    .max(SESSION_ID_MAX_LENGTH),
  revision: z.number().int().min(1),
  stage: studioStageSchema,
  brief: actorBriefSchema,
  contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
  createdAt: z.iso.datetime(),
  updatedAt: z.iso.datetime(),
}).strict();

interface StudioSessionIdentity {
  timestamp?: string;
  idSuffix?: string;
}

export function createStudioSession(
  input: ActorBrief,
  identity: StudioSessionIdentity = {},
): StudioSession {
  const brief = parseActorBrief(input);
  const timestamp = identity.timestamp ?? new Date().toISOString();
  const slug =
    brief.name
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/(^-|-$)/g, "")
      .slice(0, SESSION_SLUG_MAX_LENGTH)
      .replace(/-$/g, "") || "untitled";
  const baseId = `${slug}-${timestamp.replace(/\D/g, "").slice(0, 14)}`;

  return {
    id: identity.idSuffix ? `${baseId}-${identity.idSuffix}` : baseId,
    revision: 1,
    stage: "concept",
    brief,
    contractId: TILEFORGE_ACTOR_CONTRACT.id,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export function parseStudioSession(value: unknown): StudioSession {
  return studioSessionSchema.parse(value) as StudioSession;
}
