import * as z from "zod/v4";
import type { ActorBrief } from "./types";

export const ACTOR_BRIEF_LIMITS = {
  nameMaxLength: 80,
  descriptionMaxLength: 2_000,
} as const;

export const actorBriefInputSchema = {
  name: z
    .string()
    .trim()
    .min(1, "Name is required.")
    .max(
      ACTOR_BRIEF_LIMITS.nameMaxLength,
      `Name must be ${ACTOR_BRIEF_LIMITS.nameMaxLength} characters or fewer.`,
    ),
  kind: z.enum(["mob", "npc"]),
  description: z
    .string()
    .trim()
    .min(1, "Description is required.")
    .max(
      ACTOR_BRIEF_LIMITS.descriptionMaxLength,
      `Description must be ${ACTOR_BRIEF_LIMITS.descriptionMaxLength} characters or fewer.`,
    ),
};

export const actorBriefSchema = z.object(actorBriefInputSchema).strict();

export function parseActorBrief(value: unknown): ActorBrief {
  return actorBriefSchema.parse(value);
}

export function actorBriefError(error: unknown): string {
  if (error instanceof z.ZodError) {
    return error.issues[0]?.message ?? "The actor brief is invalid.";
  }
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  return error instanceof Error ? error.message : "The actor brief could not be saved.";
}
