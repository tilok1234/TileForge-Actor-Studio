export type StudioStage =
  | "brief"
  | "concept"
  | "turnaround"
  | "animate"
  | "world-test"
  | "export";

export type ActorKind = "mob" | "npc";

export interface ActorBrief {
  name: string;
  kind: ActorKind;
  description: string;
}

export interface BoundaryRule {
  id: string;
  label: string;
  value: string;
  severity: "locked" | "warning";
}

export interface StudioContract {
  id: string;
  version: number;
  title: string;
  sourceBoundary: string;
  frame: {
    width: number;
    height: number;
    actorHeightMin: number;
    actorHeightMax: number;
    footAnchor: [number, number];
    hardAlpha: boolean;
  };
  art: {
    lightDirection: string;
    outline: string;
    paletteMaxColors: number;
    minimumGroundLumaDistance: number;
  };
  animation: {
    directions: readonly string[];
    initialClip: string;
    framesPerDirection: number;
    frameDurationMs: number;
    groundContact: {
      mode: "foot-anchor-row";
      row: number;
    };
  };
  approval: {
    agentsMayApprove: boolean;
    immutableGenerations: boolean;
  };
}

export interface StudioSession {
  id: string;
  revision: number;
  stage: StudioStage;
  brief: ActorBrief;
  contractId: string;
  createdAt: string;
  updatedAt: string;
}
