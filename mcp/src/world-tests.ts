import { createHash, randomUUID } from "node:crypto";
import {
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { TILEFORGE_ACTOR_CONTRACT } from "../../src/lib/studio/contract.js";
import {
  parseWorldTestReferencePack,
  type WorldTestReferenceEntry,
  type WorldTestReferencePack,
} from "../../src/lib/studio/reference-pack.js";
import { TURNAROUND_DIRECTIONS } from "../../src/lib/studio/turnaround.js";
import {
  WALK_CYCLE_FRAMES_PER_DIRECTION,
} from "../../src/lib/studio/walk-cycle.js";
import {
  createWorldTestCandidateId,
  parseWorldTestCandidate,
  WORLD_TEST_ID_MAX_LENGTH,
  type WorldTestCandidate,
} from "../../src/lib/studio/world-test.js";
import {
  parseWorldTestValidationReport,
  WORLD_TEST_VALIDATOR_ID,
  type WorldTestValidationReport,
} from "../../src/lib/studio/world-test-validation.js";
import { getSession, workspaceRoot } from "./storage.js";
import {
  getWalkCycleCandidatePayload,
  type WalkCycleCandidatePayload,
} from "./walk-cycles.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const referencePackRoot = resolve(
  moduleDir,
  "../../reference-packs/tileforge-world-test-v1",
);
const referenceManifestPath = join(referencePackRoot, "manifest.json");

export interface WorldTestCandidatePayload {
  candidate: WorldTestCandidate;
  previewPngBytes: Record<string, Uint8Array>;
}
interface CreateWorldTestCandidateOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
  revision?: number;
}

interface LoadedReferencePack {
  manifest: WorldTestReferencePack;
  manifestSha256: string;
  sources: Map<string, Uint8Array>;
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function assertWorldTestId(worldTestId: string): string {
  if (
    worldTestId.length > WORLD_TEST_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(worldTestId)
  ) {
    throw new Error("Invalid World Test id.");
  }
  return worldTestId;
}

function worldTestRoot(root: string, sessionId: string): string {
  return join(root, "sessions", sessionId, "world-tests");
}

function worldTestDirectory(
  root: string,
  sessionId: string,
  worldTestId: string,
): string {
  return join(
    worldTestRoot(root, sessionId),
    assertWorldTestId(worldTestId),
  );
}

async function loadReferencePack(): Promise<LoadedReferencePack> {
  const manifestBytes = await readFile(referenceManifestPath);
  const manifest = parseWorldTestReferencePack(
    JSON.parse(manifestBytes.toString("utf8")),
  );
  const sources = new Map<string, Uint8Array>();
  for (const entry of manifest.entries) {
    const bytes = await readFile(join(referencePackRoot, entry.sourceFile));
    const decoded = PNG.sync.read(bytes, { checkCRC: true });
    if (
      bytes.byteLength !== entry.sourceByteLength ||
      sha256(bytes) !== entry.sourceSha256 ||
      decoded.width !== entry.sourceWidth ||
      decoded.height !== entry.sourceHeight
    ) {
      throw new Error(
        `Pinned reference ${entry.scene}/${entry.theme} no longer matches its manifest.`,
      );
    }
    sources.set(`${entry.scene}/${entry.theme}`, bytes);
  }
  return {
    manifest,
    manifestSha256: sha256(manifestBytes),
    sources,
  };
}

function previewFilename(entry: WorldTestReferenceEntry): string {
  return `${entry.scene}-${entry.theme}.png`;
}

function renderPreview(
  entry: WorldTestReferenceEntry,
  referenceBytes: Uint8Array,
  actorBytes: Uint8Array,
): Uint8Array {
  const source = PNG.sync.read(Buffer.from(referenceBytes), { checkCRC: true });
  const actor = PNG.sync.read(Buffer.from(actorBytes), { checkCRC: true });
  const preview = new PNG({
    width: entry.viewport.width,
    height: entry.viewport.height,
  });

  for (let y = 0; y < entry.viewport.height; y += 1) {
    for (let x = 0; x < entry.viewport.width; x += 1) {
      const sourceIndex =
        ((entry.viewport.y + y) * source.width + entry.viewport.x + x) * 4;
      const previewIndex = (y * preview.width + x) * 4;
      source.data.copy(
        preview.data,
        previewIndex,
        sourceIndex,
        sourceIndex + 4,
      );
    }
  }

  for (let y = 0; y < actor.height; y += 1) {
    for (let x = 0; x < actor.width; x += 1) {
      const actorIndex = (y * actor.width + x) * 4;
      if (actor.data[actorIndex + 3] === 0) {
        continue;
      }
      const previewIndex =
        ((entry.actorPlacement.y + y) * preview.width +
          entry.actorPlacement.x +
          x) *
        4;
      actor.data.copy(
        preview.data,
        previewIndex,
        actorIndex,
        actorIndex + 4,
      );
    }
  }

  return PNG.sync.write(preview, {
    colorType: 6,
    inputColorType: 6,
    bitDepth: 8,
  });
}

function roundedPixelLuma(red: number, green: number, blue: number): number {
  return Math.floor((299 * red + 587 * green + 114 * blue + 500) / 1000);
}

function actorMeanLuma(pngBytes: Uint8Array): number {
  const actor = PNG.sync.read(Buffer.from(pngBytes), { checkCRC: true });
  let total = 0;
  let count = 0;
  for (let index = 0; index < actor.data.length; index += 4) {
    if (actor.data[index + 3] === 0) {
      continue;
    }
    total += roundedPixelLuma(
      actor.data[index]!,
      actor.data[index + 1]!,
      actor.data[index + 2]!,
    );
    count += 1;
  }
  if (count === 0) {
    throw new Error("Walk Cycle frame has no visible actor pixels.");
  }
  return Math.floor((total + Math.floor(count / 2)) / count);
}

function groundMeanLuma(
  entry: WorldTestReferenceEntry,
  pngBytes: Uint8Array,
): number {
  const source = PNG.sync.read(Buffer.from(pngBytes), { checkCRC: true });
  let total = 0;
  let count = 0;
  for (
    let y = entry.groundSample.y;
    y < entry.groundSample.y + entry.groundSample.height;
    y += 1
  ) {
    for (
      let x = entry.groundSample.x;
      x < entry.groundSample.x + entry.groundSample.width;
      x += 1
    ) {
      const index =
        ((entry.viewport.y + y) * source.width + entry.viewport.x + x) * 4;
      total += roundedPixelLuma(
        source.data[index]!,
        source.data[index + 1]!,
        source.data[index + 2]!,
      );
      count += 1;
    }
  }
  return Math.floor((total + Math.floor(count / 2)) / count);
}

function assertSourceWalkCycle(
  candidate: WorldTestCandidate,
  source: WalkCycleCandidatePayload,
): void {
  if (candidate.sourceWalkCycle.walkCycleId !== source.candidate.id) {
    throw new Error("World Test source Walk Cycle identity changed.");
  }
  for (const [index, frame] of source.candidate.frames.entries()) {
    const receipt = candidate.sourceWalkCycle.frameSources[index];
    if (
      receipt?.direction !== frame.direction ||
      receipt.frameIndex !== frame.frameIndex ||
      receipt.sha256 !== frame.sha256 ||
      receipt.byteLength !== frame.byteLength
    ) {
      throw new Error("World Test source Walk Cycle bytes changed.");
    }
  }
}

export async function listWorldTestCandidates(
  sessionId: string,
  root = workspaceRoot,
): Promise<WorldTestCandidate[]> {
  const session = await getSession(sessionId, root);
  const rootDirectory = worldTestRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const entries = await readdir(rootDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getWorldTestCandidate(session.id, entry.name, root);
        } catch {
          return null;
        }
      }),
  );
  return candidates
    .filter((candidate): candidate is WorldTestCandidate => candidate !== null)
    .sort((left, right) => right.revision - left.revision);
}

export async function getWorldTestCandidate(
  sessionId: string,
  worldTestId: string,
  root = workspaceRoot,
): Promise<WorldTestCandidate> {
  const session = await getSession(sessionId, root);
  const raw = await readFile(
    join(
      worldTestDirectory(root, session.id, worldTestId),
      "world-test.json",
    ),
    "utf8",
  );
  const candidate = parseWorldTestCandidate(JSON.parse(raw));
  if (candidate.sessionId !== session.id || candidate.id !== worldTestId) {
    throw new Error("World Test identity does not match its storage path.");
  }
  return candidate;
}

export async function getWorldTestCandidatePayload(
  sessionId: string,
  worldTestId: string,
  root = workspaceRoot,
): Promise<WorldTestCandidatePayload> {
  const candidate = await getWorldTestCandidate(
    sessionId,
    worldTestId,
    root,
  );
  const directory = worldTestDirectory(root, sessionId, worldTestId);
  const previewPngBytes: Record<string, Uint8Array> = {};
  for (const preview of candidate.previews) {
    const bytes = await readFile(join(directory, preview.sourceFile));
    const decoded = PNG.sync.read(bytes, { checkCRC: true });
    if (
      bytes.byteLength !== preview.byteLength ||
      sha256(bytes) !== preview.sha256 ||
      decoded.width !== preview.width ||
      decoded.height !== preview.height
    ) {
      throw new Error(
        `${preview.scene}/${preview.theme} preview no longer matches immutable provenance.`,
      );
    }
    previewPngBytes[`${preview.scene}/${preview.theme}`] = bytes;
  }
  return { candidate, previewPngBytes };
}

export async function createWorldTestCandidate(
  sessionId: string,
  sourceWalkCycleId: string,
  options: CreateWorldTestCandidateOptions = {},
): Promise<WorldTestCandidate> {
  const root = options.root ?? workspaceRoot;
  const session = await getSession(sessionId, root);
  const sourceWalkCycle = await getWalkCycleCandidatePayload(
    session.id,
    sourceWalkCycleId,
    root,
  );
  const referencePack = await loadReferencePack();
  const existing = await listWorldTestCandidates(session.id, root);
  const revision =
    options.revision ??
    existing.reduce(
      (highest, candidate) => Math.max(highest, candidate.revision),
      0,
    ) + 1;
  const timestamp = options.timestamp ?? new Date().toISOString();
  const previewBytes = new Map<string, Uint8Array>();
  const downFrameZero = sourceWalkCycle.pngBytes.down[0];
  if (!downFrameZero) {
    throw new Error("Accepted Walk Cycle down frame 0 is unavailable.");
  }
  for (const entry of referencePack.manifest.entries) {
    const referenceBytes = referencePack.sources.get(
      `${entry.scene}/${entry.theme}`,
    );
    if (!referenceBytes) {
      throw new Error("Pinned reference pack is incomplete.");
    }
    previewBytes.set(
      `${entry.scene}/${entry.theme}`,
      renderPreview(entry, referenceBytes, downFrameZero),
    );
  }

  const candidate = parseWorldTestCandidate({
    schemaVersion: 1,
    id: createWorldTestCandidateId(
      revision,
      timestamp,
      options.idSuffix ?? randomUUID().slice(0, 8),
    ),
    revision,
    sessionId: session.id,
    stage: "world-test",
    contractId: session.contractId,
    sourceWalkCycle: {
      walkCycleId: sourceWalkCycle.candidate.id,
      frameSources: sourceWalkCycle.candidate.frames.map((frame) => ({
        direction: frame.direction,
        frameIndex: frame.frameIndex,
        sha256: frame.sha256,
        byteLength: frame.byteLength,
      })),
      acceptedBy: "user",
      acceptedAt: timestamp,
    },
    referencePack: {
      id: referencePack.manifest.id,
      version: referencePack.manifest.version,
      manifestSha256: referencePack.manifestSha256,
      checkoutCommit: referencePack.manifest.source.checkoutCommit,
      generatedEngineCommit:
        referencePack.manifest.source.generatedEngineCommit,
    },
    previews: referencePack.manifest.entries.map((entry) => {
      const bytes = previewBytes.get(`${entry.scene}/${entry.theme}`)!;
      return {
        scene: entry.scene,
        theme: entry.theme,
        sourceFile: previewFilename(entry),
        sha256: sha256(bytes),
        byteLength: bytes.byteLength,
        width: referencePack.manifest.preview.width,
        height: referencePack.manifest.preview.height,
        referenceSourceSha256: entry.sourceSha256,
      };
    }),
    createdAt: timestamp,
    preparation: {
      method: "local-deterministic-compositor-v1",
      additionalAiCost: false,
    },
    reviewStatus: "unreviewed",
    finalArtJudgment: {
      status: "not_assessed",
      authority: "user",
      message:
        "Only the user can approve final art after reviewing World Test evidence.",
    },
  });

  const candidatesRoot = worldTestRoot(root, session.id);
  await mkdir(candidatesRoot, { recursive: true });
  const finalDirectory = worldTestDirectory(root, session.id, candidate.id);
  const temporaryDirectory = join(
    candidatesRoot,
    `.${candidate.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });
  try {
    for (const preview of candidate.previews) {
      const bytes = previewBytes.get(`${preview.scene}/${preview.theme}`);
      if (!bytes) {
        throw new Error("World Test preview bytes are incomplete.");
      }
      await writeFile(join(temporaryDirectory, preview.sourceFile), bytes, {
        flag: "wx",
      });
    }
    await writeFile(
      join(temporaryDirectory, "world-test.json"),
      `${JSON.stringify(candidate, null, 2)}\n`,
      { encoding: "utf8", flag: "wx" },
    );
    await rename(temporaryDirectory, finalDirectory);
    return candidate;
  } catch (error) {
    await rm(temporaryDirectory, { recursive: true, force: true });
    throw error;
  }
}

export async function validateWorldTestCandidate(
  sessionId: string,
  worldTestId: string,
  root = workspaceRoot,
): Promise<WorldTestValidationReport> {
  const payload = await getWorldTestCandidatePayload(
    sessionId,
    worldTestId,
    root,
  );
  const sourceWalkCycle = await getWalkCycleCandidatePayload(
    sessionId,
    payload.candidate.sourceWalkCycle.walkCycleId,
    root,
  );
  assertSourceWalkCycle(payload.candidate, sourceWalkCycle);
  const referencePack = await loadReferencePack();
  if (
    payload.candidate.referencePack.manifestSha256 !==
      referencePack.manifestSha256 ||
    payload.candidate.referencePack.checkoutCommit !==
      referencePack.manifest.source.checkoutCommit ||
    payload.candidate.referencePack.generatedEngineCommit !==
      referencePack.manifest.source.generatedEngineCommit
  ) {
    throw new Error("World Test reference-pack receipt no longer matches.");
  }

  const actorLumas = new Map<string, number>();
  for (const source of sourceWalkCycle.candidate.frames) {
    actorLumas.set(
      `${source.direction}/${source.frameIndex}`,
      actorMeanLuma(
        sourceWalkCycle.pngBytes[source.direction][source.frameIndex]!,
      ),
    );
  }
  const measurements = referencePack.manifest.entries.flatMap((entry) => {
    const referenceBytes = referencePack.sources.get(
      `${entry.scene}/${entry.theme}`,
    )!;
    const ground = groundMeanLuma(entry, referenceBytes);
    return TURNAROUND_DIRECTIONS.flatMap((direction) =>
      Array.from(
        { length: WALK_CYCLE_FRAMES_PER_DIRECTION },
        (_, frameIndex) => {
          const actor = actorLumas.get(`${direction}/${frameIndex}`)!;
          const distance = Math.abs(actor - ground);
          return {
            scene: entry.scene,
            theme: entry.theme,
            direction,
            frameIndex,
            actorMeanLuma: actor,
            groundMeanLuma: ground,
            distance,
            minimum:
              TILEFORGE_ACTOR_CONTRACT.art.minimumGroundLumaDistance,
            status:
              distance >=
              TILEFORGE_ACTOR_CONTRACT.art.minimumGroundLumaDistance
                ? ("pass" as const)
                : ("fail" as const),
          };
        },
      ),
    );
  });
  const summary = measurements.reduce(
    (counts, measurement) => {
      counts[measurement.status] += 1;
      return counts;
    },
    { pass: 0, fail: 0, notAssessed: 0 as const },
  );
  return parseWorldTestValidationReport({
    schemaVersion: 1,
    validatorId: WORLD_TEST_VALIDATOR_ID,
    worldTestId: payload.candidate.id,
    contractId: payload.candidate.contractId,
    measurements,
    summary,
    finalArtJudgment: payload.candidate.finalArtJudgment,
  });
}
