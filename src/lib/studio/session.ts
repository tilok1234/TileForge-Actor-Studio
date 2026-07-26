import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import type { ActorBrief, StudioSession } from "./types";

export function createStudioSession(brief: ActorBrief): StudioSession {
  const timestamp = new Date().toISOString();
  const slug =
    brief.name
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/(^-|-$)/g, "") || "untitled";

  return {
    id: `${slug}-${timestamp.replace(/\D/g, "").slice(0, 14)}`,
    revision: 1,
    stage: "brief",
    brief,
    contractId: TILEFORGE_ACTOR_CONTRACT.id,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}
