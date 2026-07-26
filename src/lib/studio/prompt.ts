import { TILEFORGE_ACTOR_CONTRACT as contract } from "./contract";
import type { ActorBrief } from "./types";

export function compileActorPrompt(brief: ActorBrief): string {
  const identity = brief.name.trim() || "Unnamed actor";
  const description =
    brief.description.trim() || "A distinctive top-down fantasy actor.";

  return [
    `Create one ${contract.frame.width}x${contract.frame.height} pixel-art ${brief.kind} named "${identity}".`,
    description,
    "",
    "Locked world contract:",
    `- The visible actor is ${contract.frame.actorHeightMin}-${contract.frame.actorHeightMax}px tall.`,
    `- Feet remain anchored at (${contract.frame.footAnchor.join(", ")}).`,
    `- Lighting comes from the ${contract.art.lightDirection}.`,
    `- Use ${contract.art.outline}.`,
    `- Use at most ${contract.art.paletteMaxColors} colors with hard transparent pixels.`,
    `- Preserve at least ${contract.art.minimumGroundLumaDistance} luma separation from walkable TileForge grounds.`,
    "- Render a single down-facing approval concept. Do not create a full sheet yet.",
    "- No background, text, border, mockup, or soft antialiasing.",
    "- Return a candidate only. Only the user may approve final art.",
  ].join("\n");
}
