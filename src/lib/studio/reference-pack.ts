import * as z from "zod/v4";
import { candidateSha256Schema } from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";

export const WORLD_TEST_REFERENCE_PACK_ID = "tileforge-world-test-v1";
export const WORLD_TEST_SCENES = [
  "scale-lineup",
  "forest-clearing",
  "crownhold",
  "tidewater",
] as const;
export const WORLD_TEST_THEMES = [
  "forest",
  "autumn",
  "dusk",
  "winter",
] as const;
export const WORLD_TEST_PREVIEW_WIDTH = 640;
export const WORLD_TEST_PREVIEW_HEIGHT = 384;

const rectangleSchema = z
  .object({
    x: z.number().int().min(0),
    y: z.number().int().min(0),
    width: z.number().int().min(1),
    height: z.number().int().min(1),
  })
  .strict();

const pointSchema = z
  .object({
    x: z.number().int().min(0),
    y: z.number().int().min(0),
  })
  .strict();

const referencePackEntrySchema = z
  .object({
    scene: z.enum(WORLD_TEST_SCENES),
    theme: z.enum(WORLD_TEST_THEMES),
    sourceFile: z
      .string()
      .regex(
        /^images\/[a-z0-9-]+\.png$/,
        "Reference source must be a pack-local PNG.",
      ),
    sourceSha256: candidateSha256Schema,
    sourceByteLength: z.number().int().min(1),
    sourceWidth: z.number().int().min(WORLD_TEST_PREVIEW_WIDTH),
    sourceHeight: z.number().int().min(WORLD_TEST_PREVIEW_HEIGHT),
    viewport: rectangleSchema,
    actorPlacement: pointSchema,
    groundSample: rectangleSchema,
  })
  .strict()
  .superRefine((entry, context) => {
    if (
      entry.viewport.width !== WORLD_TEST_PREVIEW_WIDTH ||
      entry.viewport.height !== WORLD_TEST_PREVIEW_HEIGHT ||
      entry.viewport.x + entry.viewport.width > entry.sourceWidth ||
      entry.viewport.y + entry.viewport.height > entry.sourceHeight
    ) {
      context.addIssue({
        code: "custom",
        path: ["viewport"],
        message: "Reference viewport must be an in-bounds 640 x 384 crop.",
      });
    }
    if (
      entry.actorPlacement.x + TILEFORGE_ACTOR_CONTRACT.frame.width >
        entry.viewport.width ||
      entry.actorPlacement.y + TILEFORGE_ACTOR_CONTRACT.frame.height >
        entry.viewport.height
    ) {
      context.addIssue({
        code: "custom",
        path: ["actorPlacement"],
        message: "Actor placement must fit inside the reference viewport.",
      });
    }
    if (
      entry.groundSample.x + entry.groundSample.width >
        entry.viewport.width ||
      entry.groundSample.y + entry.groundSample.height >
        entry.viewport.height
    ) {
      context.addIssue({
        code: "custom",
        path: ["groundSample"],
        message: "Ground sample must fit inside the reference viewport.",
      });
    }
  });

export const worldTestReferencePackSchema = z
  .object({
    schemaVersion: z.literal(1),
    id: z.literal(WORLD_TEST_REFERENCE_PACK_ID),
    version: z.literal(1),
    contractId: z.literal(TILEFORGE_ACTOR_CONTRACT.id),
    source: z
      .object({
        repository: z.string().min(1),
        checkoutCommit: z.string().regex(/^[a-f0-9]{40}$/),
        generatedEngineCommit: z.string().regex(/^[a-f0-9]{7,40}$/),
        generated: z.string().min(1),
        renderPath: z.string().min(1),
        scale: z.literal("1x"),
      })
      .strict(),
    preview: z
      .object({
        width: z.literal(WORLD_TEST_PREVIEW_WIDTH),
        height: z.literal(WORLD_TEST_PREVIEW_HEIGHT),
        actorDirection: z.literal("down"),
        actorFrameIndex: z.literal(0),
        compositor: z.literal("nearest-neighbor-hard-alpha-v1"),
      })
      .strict(),
    entries: z
      .array(referencePackEntrySchema)
      .length(WORLD_TEST_SCENES.length * WORLD_TEST_THEMES.length),
  })
  .strict()
  .superRefine((pack, context) => {
    for (const [sceneIndex, scene] of WORLD_TEST_SCENES.entries()) {
      for (const [themeIndex, theme] of WORLD_TEST_THEMES.entries()) {
        const index = sceneIndex * WORLD_TEST_THEMES.length + themeIndex;
        const entry = pack.entries[index];
        if (entry?.scene !== scene || entry.theme !== theme) {
          context.addIssue({
            code: "custom",
            path: ["entries", index],
            message:
              "Reference entries must use canonical scene-major, theme-minor order.",
          });
        }
      }
    }
  });

export type WorldTestReferencePack = z.infer<
  typeof worldTestReferencePackSchema
>;
export type WorldTestReferenceEntry = z.infer<
  typeof referencePackEntrySchema
>;
export type WorldTestScene = (typeof WORLD_TEST_SCENES)[number];
export type WorldTestTheme = (typeof WORLD_TEST_THEMES)[number];

export function parseWorldTestReferencePack(
  value: unknown,
): WorldTestReferencePack {
  return worldTestReferencePackSchema.parse(value);
}
