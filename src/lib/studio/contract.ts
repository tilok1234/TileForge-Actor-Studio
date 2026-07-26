import contractData from "../../../contracts/tileforge-actor-32-v1.json";
import type { BoundaryRule, StudioContract } from "./types";

export const TILEFORGE_ACTOR_CONTRACT = contractData as unknown as StudioContract;

export const BOUNDARY_RULES: BoundaryRule[] = [
  {
    id: "canvas",
    label: "Canvas",
    value: "32 × 32 px, transparent",
    severity: "locked",
  },
  {
    id: "height",
    label: "Actor height",
    value: "22–30 px",
    severity: "locked",
  },
  {
    id: "anchor",
    label: "Foot anchor",
    value: "16, 28",
    severity: "locked",
  },
  {
    id: "light",
    label: "Lighting",
    value: "North-west",
    severity: "locked",
  },
  {
    id: "palette",
    label: "Palette",
    value: "Maximum 16 colors",
    severity: "locked",
  },
  {
    id: "contrast",
    label: "Ground contrast",
    value: "15+ luma separation",
    severity: "warning",
  },
];
