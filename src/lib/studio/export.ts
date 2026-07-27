import * as z from "zod/v4";
import { candidateSha256Schema } from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  WORLD_TEST_SCENES,
  WORLD_TEST_THEMES,
} from "./reference-pack";
import { SESSION_ID_MAX_LENGTH } from "./session";
import { TURNAROUND_DIRECTIONS } from "./turnaround";
import {
  WALK_CYCLE_FRAME_DURATION_MS,
  WALK_CYCLE_FRAMES_PER_DIRECTION,
  walkCycleIdSchema,
} from "./walk-cycle";
import {
  type WorldTestCandidate,
  worldTestIdSchema,
} from "./world-test";

export const EXPORT_DOCUMENT_VERSION = 1;
export const EXPORT_METADATA_VERSION = 1;
export const EXPORT_PROVENANCE_VERSION = 1;
export const EXPORT_ID_MAX_LENGTH = 96;
export const EXPORT_SHEET_FILE = "sprite-sheet.png";
export const EXPORT_METADATA_FILE = "metadata.json";
export const EXPORT_PROVENANCE_FILE = "provenance.json";
export const EXPORT_SHEET_WIDTH =
  TILEFORGE_ACTOR_CONTRACT.frame.width * WALK_CYCLE_FRAMES_PER_DIRECTION;
export const EXPORT_SHEET_HEIGHT =
  TILEFORGE_ACTOR_CONTRACT.frame.height * TURNAROUND_DIRECTIONS.length;
export const EXPORT_SHEET_LAYOUT = "direction-rows-frame-columns-v1";

export const exportIdSchema = z
  .string()
  .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid Export id.")
  .max(EXPORT_ID_MAX_LENGTH);

const frameSourceSchema = z
  .object({
    direction: z.enum(TURNAROUND_DIRECTIONS),
    frameIndex: z
      .number()
      .int()
      .min(0)
      .max(WALK_CYCLE_FRAMES_PER_DIRECTION - 1),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
  })
  .strict();

const previewSourceSchema = z
  .object({
    scene: z.enum(WORLD_TEST_SCENES),
    theme: z.enum(WORLD_TEST_THEMES),
    sourceFile: z.string().regex(/^[a-z0-9-]+\.png$/),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
  })
  .strict();

export const approvedWorldTestReceiptSchema = z
  .object({
    worldTestId: worldTestIdSchema,
    documentSha256: candidateSha256Schema,
    previewSources: z
      .array(previewSourceSchema)
      .length(WORLD_TEST_SCENES.length * WORLD_TEST_THEMES.length),
    approvedBy: z.literal("user"),
    approvedAt: z.iso.datetime(),
  })
  .strict()
  .superRefine((receipt, context) => {
    for (const [sceneIndex, scene] of WORLD_TEST_SCENES.entries()) {
      for (const [themeIndex, theme] of WORLD_TEST_THEMES.entries()) {
        const index = sceneIndex * WORLD_TEST_THEMES.length + themeIndex;
        const preview = receipt.previewSources[index];
        if (
          preview?.scene !== scene ||
          preview.theme !== theme ||
          preview.sourceFile !== `${scene}-${theme}.png`
        ) {
          context.addIssue({
            code: "custom",
            path: ["previewSources", index],
            message:
              "Approved World Test previews must use canonical scene and theme order.",
          });
        }
      }
    }
  });

export const exportSourceWalkCycleSchema = z
  .object({
    walkCycleId: walkCycleIdSchema,
    frameSources: z
      .array(frameSourceSchema)
      .length(
        TURNAROUND_DIRECTIONS.length * WALK_CYCLE_FRAMES_PER_DIRECTION,
      ),
  })
  .strict();

export const exportPreparationSchema = z
  .object({
    method: z.literal("local-deterministic-sheet-v1"),
    additionalAiCost: z.literal(false),
  })
  .strict();

export const publishingBoundarySchema = z
  .object({
    status: z.literal("not_approved"),
    authority: z.literal("user"),
    message: z.string().min(1),
  })
  .strict();

const fileReceiptSchema = z
  .object({
    sourceFile: z.string().min(1),
    sha256: candidateSha256Schema,
    byteLength: z.number().int().min(1),
  })
  .strict();

const sheetReceiptSchema = fileReceiptSchema.extend({
  sourceFile: z.literal(EXPORT_SHEET_FILE),
  width: z.literal(EXPORT_SHEET_WIDTH),
  height: z.literal(EXPORT_SHEET_HEIGHT),
  cellWidth: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
  cellHeight: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
  layout: z.literal(EXPORT_SHEET_LAYOUT),
});

const canonicalFrameOrder = (
  frames: ReadonlyArray<{ direction: string; frameIndex: number }>,
  context: z.RefinementCtx,
  path: Array<string | number>,
) => {
  for (const [directionIndex, direction] of TURNAROUND_DIRECTIONS.entries()) {
    for (
      let frameIndex = 0;
      frameIndex < WALK_CYCLE_FRAMES_PER_DIRECTION;
      frameIndex += 1
    ) {
      const index =
        directionIndex * WALK_CYCLE_FRAMES_PER_DIRECTION + frameIndex;
      const frame = frames[index];
      if (
        frame?.direction !== direction ||
        frame.frameIndex !== frameIndex
      ) {
        context.addIssue({
          code: "custom",
          path: [...path, index],
          message:
            "Export frames must use canonical direction and frame order.",
        });
      }
    }
  }
};

export const exportMetadataSchema = z
  .object({
    schemaVersion: z.literal(EXPORT_METADATA_VERSION),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    actor: z
      .object({
        name: z.string().min(1).max(80),
        kind: z.enum(["mob", "npc"]),
      })
      .strict(),
    sheet: z
      .object({
        sourceFile: z.literal(EXPORT_SHEET_FILE),
        width: z.literal(EXPORT_SHEET_WIDTH),
        height: z.literal(EXPORT_SHEET_HEIGHT),
        cellWidth: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
        cellHeight: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
        layout: z.literal(EXPORT_SHEET_LAYOUT),
      })
      .strict(),
    animation: z
      .object({
        clip: z.literal(TILEFORGE_ACTOR_CONTRACT.animation.initialClip),
        directions: z
          .array(z.enum(TURNAROUND_DIRECTIONS))
          .length(TURNAROUND_DIRECTIONS.length),
        framesPerDirection: z.literal(WALK_CYCLE_FRAMES_PER_DIRECTION),
        frameDurationMs: z.literal(WALK_CYCLE_FRAME_DURATION_MS),
        footAnchor: z.tuple([
          z.literal(TILEFORGE_ACTOR_CONTRACT.frame.footAnchor[0]),
          z.literal(TILEFORGE_ACTOR_CONTRACT.frame.footAnchor[1]),
        ]),
      })
      .strict(),
    frames: z
      .array(
        frameSourceSchema.extend({
          x: z.number().int().min(0).max(EXPORT_SHEET_WIDTH - 1),
          y: z.number().int().min(0).max(EXPORT_SHEET_HEIGHT - 1),
          width: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.width),
          height: z.literal(TILEFORGE_ACTOR_CONTRACT.frame.height),
        }),
      )
      .length(
        TURNAROUND_DIRECTIONS.length * WALK_CYCLE_FRAMES_PER_DIRECTION,
      ),
  })
  .strict()
  .superRefine((metadata, context) => {
    if (
      metadata.animation.directions.some(
        (direction, index) => direction !== TURNAROUND_DIRECTIONS[index],
      )
    ) {
      context.addIssue({
        code: "custom",
        path: ["animation", "directions"],
        message: "Export directions must use canonical contract order.",
      });
    }
    canonicalFrameOrder(metadata.frames, context, ["frames"]);
    for (const [index, frame] of metadata.frames.entries()) {
      const directionIndex = Math.floor(
        index / WALK_CYCLE_FRAMES_PER_DIRECTION,
      );
      if (
        frame.x !==
          frame.frameIndex * TILEFORGE_ACTOR_CONTRACT.frame.width ||
        frame.y !== directionIndex * TILEFORGE_ACTOR_CONTRACT.frame.height
      ) {
        context.addIssue({
          code: "custom",
          path: ["frames", index],
          message: "Export frame coordinates do not match the sheet layout.",
        });
      }
    }
  });

export const exportProvenanceSchema = z
  .object({
    schemaVersion: z.literal(EXPORT_PROVENANCE_VERSION),
    exportId: exportIdSchema,
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    approvedWorldTest: approvedWorldTestReceiptSchema,
    sourceWalkCycle: exportSourceWalkCycleSchema,
    preparation: exportPreparationSchema,
    publishing: publishingBoundarySchema,
  })
  .strict()
  .superRefine((provenance, context) => {
    canonicalFrameOrder(
      provenance.sourceWalkCycle.frameSources,
      context,
      ["sourceWalkCycle", "frameSources"],
    );
  });

export const exportCandidateSchema = z
  .object({
    schemaVersion: z.literal(EXPORT_DOCUMENT_VERSION),
    id: exportIdSchema,
    revision: z.number().int().min(1),
    sessionId: z
      .string()
      .regex(/^[a-z0-9][a-z0-9-]{2,95}$/i, "Invalid session id.")
      .max(SESSION_ID_MAX_LENGTH),
    stage: z.literal("export"),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    approvedWorldTest: approvedWorldTestReceiptSchema,
    sourceWalkCycle: exportSourceWalkCycleSchema,
    package: z
      .object({
        spriteSheet: sheetReceiptSchema,
        metadata: fileReceiptSchema.extend({
          sourceFile: z.literal(EXPORT_METADATA_FILE),
        }),
        provenance: fileReceiptSchema.extend({
          sourceFile: z.literal(EXPORT_PROVENANCE_FILE),
        }),
      })
      .strict(),
    createdAt: z.iso.datetime(),
    preparation: exportPreparationSchema,
    status: z.literal("draft"),
    publishing: publishingBoundarySchema,
  })
  .strict()
  .superRefine((candidate, context) => {
    canonicalFrameOrder(
      candidate.sourceWalkCycle.frameSources,
      context,
      ["sourceWalkCycle", "frameSources"],
    );
  });

export type ApprovedWorldTestReceipt = z.infer<
  typeof approvedWorldTestReceiptSchema
>;
export type ExportSourceWalkCycle = z.infer<
  typeof exportSourceWalkCycleSchema
>;
export type ExportMetadata = z.infer<typeof exportMetadataSchema>;
export type ExportProvenance = z.infer<typeof exportProvenanceSchema>;
export type ExportCandidate = z.infer<typeof exportCandidateSchema>;

export function parseExportMetadata(value: unknown): ExportMetadata {
  return exportMetadataSchema.parse(value);
}

export function parseExportProvenance(value: unknown): ExportProvenance {
  return exportProvenanceSchema.parse(value);
}

export function parseExportCandidate(value: unknown): ExportCandidate {
  return exportCandidateSchema.parse(value);
}

export function createExportCandidateId(
  revision: number,
  timestamp: string,
  idSuffix: string,
): string {
  const paddedRevision = revision.toString().padStart(4, "0");
  const timestampDigits = timestamp.replace(/\D/g, "").slice(0, 14);
  return exportIdSchema.parse(
    `export-r${paddedRevision}-${timestampDigits}-${idSuffix}`,
  );
}

export function worldTestPreviewReceipt(
  candidate: WorldTestCandidate,
): ApprovedWorldTestReceipt["previewSources"] {
  return candidate.previews.map((preview) => ({
    scene: preview.scene,
    theme: preview.theme,
    sourceFile: preview.sourceFile,
    sha256: preview.sha256,
    byteLength: preview.byteLength,
  }));
}
