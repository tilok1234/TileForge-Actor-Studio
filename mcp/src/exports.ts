import { createHash, randomUUID } from "node:crypto";
import {
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import { PNG } from "pngjs";
import { TILEFORGE_ACTOR_CONTRACT } from "../../src/lib/studio/contract.js";
import {
  createExportCandidateId,
  EXPORT_ID_MAX_LENGTH,
  EXPORT_METADATA_FILE,
  EXPORT_PROVENANCE_FILE,
  EXPORT_SHEET_FILE,
  EXPORT_SHEET_HEIGHT,
  EXPORT_SHEET_LAYOUT,
  EXPORT_SHEET_WIDTH,
  parseExportCandidate,
  parseExportMetadata,
  parseExportProvenance,
  worldTestPreviewReceipt,
  type ExportCandidate,
  type ExportMetadata,
  type ExportProvenance,
} from "../../src/lib/studio/export.js";
import {
  EXPORT_VALIDATION_CHECKS,
  EXPORT_VALIDATOR_ID,
  parseExportValidationReport,
  type ExportValidationReport,
} from "../../src/lib/studio/export-validation.js";
import { TURNAROUND_DIRECTIONS } from "../../src/lib/studio/turnaround.js";
import {
  WALK_CYCLE_FRAME_DURATION_MS,
  WALK_CYCLE_FRAMES_PER_DIRECTION,
} from "../../src/lib/studio/walk-cycle.js";
import { getSession, workspaceRoot } from "./storage.js";
import {
  getWalkCycleCandidatePayload,
  type WalkCycleCandidatePayload,
} from "./walk-cycles.js";
import {
  getWorldTestCandidatePayload,
  type WorldTestCandidatePayload,
} from "./world-tests.js";

export interface ExportCandidatePayload {
  candidate: ExportCandidate;
  spriteSheetPngBytes: Uint8Array;
  metadata: ExportMetadata;
  provenance: ExportProvenance;
}

interface CreateExportCandidateOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
  revision?: number;
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function jsonBytes(value: unknown): Uint8Array {
  return Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function assertExportId(exportId: string): string {
  if (
    exportId.length > EXPORT_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(exportId)
  ) {
    throw new Error("Invalid Export id.");
  }
  return exportId;
}

function exportRoot(root: string, sessionId: string): string {
  return join(root, "sessions", sessionId, "exports");
}

function exportDirectory(
  root: string,
  sessionId: string,
  exportId: string,
): string {
  return join(exportRoot(root, sessionId), assertExportId(exportId));
}

function sourceWorldTestDocumentPath(
  root: string,
  sessionId: string,
  worldTestId: string,
): string {
  return join(
    root,
    "sessions",
    sessionId,
    "world-tests",
    worldTestId,
    "world-test.json",
  );
}

function assertWorldTestReceipt(
  candidate: ExportCandidate,
  worldTest: WorldTestCandidatePayload,
  documentBytes: Uint8Array,
): void {
  const receipt = candidate.approvedWorldTest;
  if (
    receipt.worldTestId !== worldTest.candidate.id ||
    receipt.documentSha256 !== sha256(documentBytes) ||
    JSON.stringify(receipt.previewSources) !==
      JSON.stringify(worldTestPreviewReceipt(worldTest.candidate))
  ) {
    throw new Error("Export approved World Test receipt no longer matches.");
  }
}

function assertSourceWalkCycle(
  candidate: ExportCandidate,
  worldTest: WorldTestCandidatePayload,
  walkCycle: WalkCycleCandidatePayload,
): void {
  if (
    candidate.sourceWalkCycle.walkCycleId !== walkCycle.candidate.id ||
    worldTest.candidate.sourceWalkCycle.walkCycleId !== walkCycle.candidate.id
  ) {
    throw new Error("Export source Walk Cycle identity changed.");
  }
  for (const [index, frame] of walkCycle.candidate.frames.entries()) {
    const exportReceipt = candidate.sourceWalkCycle.frameSources[index];
    const worldTestReceipt =
      worldTest.candidate.sourceWalkCycle.frameSources[index];
    if (
      exportReceipt?.direction !== frame.direction ||
      exportReceipt.frameIndex !== frame.frameIndex ||
      exportReceipt.sha256 !== frame.sha256 ||
      exportReceipt.byteLength !== frame.byteLength ||
      worldTestReceipt?.direction !== frame.direction ||
      worldTestReceipt.frameIndex !== frame.frameIndex ||
      worldTestReceipt.sha256 !== frame.sha256 ||
      worldTestReceipt.byteLength !== frame.byteLength
    ) {
      throw new Error("Export source Walk Cycle bytes changed.");
    }
  }
}

function renderSpriteSheet(source: WalkCycleCandidatePayload): Uint8Array {
  const sheet = new PNG({
    width: EXPORT_SHEET_WIDTH,
    height: EXPORT_SHEET_HEIGHT,
  });
  sheet.data.fill(0);
  for (const [directionIndex, direction] of TURNAROUND_DIRECTIONS.entries()) {
    for (
      let frameIndex = 0;
      frameIndex < WALK_CYCLE_FRAMES_PER_DIRECTION;
      frameIndex += 1
    ) {
      const bytes = source.pngBytes[direction][frameIndex];
      if (!bytes) {
        throw new Error("Export source frame set is incomplete.");
      }
      const frame = PNG.sync.read(Buffer.from(bytes), { checkCRC: true });
      if (
        frame.width !== TILEFORGE_ACTOR_CONTRACT.frame.width ||
        frame.height !== TILEFORGE_ACTOR_CONTRACT.frame.height
      ) {
        throw new Error("Export source frame dimensions are incompatible.");
      }
      for (let y = 0; y < frame.height; y += 1) {
        const sourceStart = y * frame.width * 4;
        const targetStart =
          ((directionIndex * frame.height + y) * sheet.width +
            frameIndex * frame.width) *
          4;
        frame.data.copy(
          sheet.data,
          targetStart,
          sourceStart,
          sourceStart + frame.width * 4,
        );
      }
    }
  }
  return PNG.sync.write(sheet, {
    colorType: 6,
    inputColorType: 6,
    bitDepth: 8,
  });
}

function buildMetadata(
  actor: { name: string; kind: "mob" | "npc" },
  source: WalkCycleCandidatePayload,
): ExportMetadata {
  return parseExportMetadata({
    schemaVersion: 1,
    contractId: TILEFORGE_ACTOR_CONTRACT.id,
    actor,
    sheet: {
      sourceFile: EXPORT_SHEET_FILE,
      width: EXPORT_SHEET_WIDTH,
      height: EXPORT_SHEET_HEIGHT,
      cellWidth: TILEFORGE_ACTOR_CONTRACT.frame.width,
      cellHeight: TILEFORGE_ACTOR_CONTRACT.frame.height,
      layout: EXPORT_SHEET_LAYOUT,
    },
    animation: {
      clip: TILEFORGE_ACTOR_CONTRACT.animation.initialClip,
      directions: [...TURNAROUND_DIRECTIONS],
      framesPerDirection: WALK_CYCLE_FRAMES_PER_DIRECTION,
      frameDurationMs: WALK_CYCLE_FRAME_DURATION_MS,
      footAnchor: [...TILEFORGE_ACTOR_CONTRACT.frame.footAnchor],
    },
    frames: source.candidate.frames.map((frame, index) => ({
      direction: frame.direction,
      frameIndex: frame.frameIndex,
      x: frame.frameIndex * TILEFORGE_ACTOR_CONTRACT.frame.width,
      y:
        Math.floor(index / WALK_CYCLE_FRAMES_PER_DIRECTION) *
        TILEFORGE_ACTOR_CONTRACT.frame.height,
      width: TILEFORGE_ACTOR_CONTRACT.frame.width,
      height: TILEFORGE_ACTOR_CONTRACT.frame.height,
      sha256: frame.sha256,
      byteLength: frame.byteLength,
    })),
  });
}

function buildProvenance(
  exportId: string,
  sessionId: string,
  approvedWorldTest: ExportCandidate["approvedWorldTest"],
  sourceWalkCycle: ExportCandidate["sourceWalkCycle"],
): ExportProvenance {
  return parseExportProvenance({
    schemaVersion: 1,
    exportId,
    sessionId,
    approvedWorldTest,
    sourceWalkCycle,
    preparation: {
      method: "local-deterministic-sheet-v1",
      additionalAiCost: false,
    },
    publishing: {
      status: "not_approved",
      authority: "user",
      message:
        "This draft export is local only. Publishing requires a separate explicit user decision.",
    },
  });
}

export async function listExportCandidates(
  sessionId: string,
  root = workspaceRoot,
): Promise<ExportCandidate[]> {
  const session = await getSession(sessionId, root);
  const rootDirectory = exportRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const entries = await readdir(rootDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getExportCandidate(session.id, entry.name, root);
        } catch {
          return null;
        }
      }),
  );
  return candidates
    .filter((candidate): candidate is ExportCandidate => candidate !== null)
    .sort((left, right) => right.revision - left.revision);
}

export async function getExportCandidate(
  sessionId: string,
  exportId: string,
  root = workspaceRoot,
): Promise<ExportCandidate> {
  const session = await getSession(sessionId, root);
  const raw = await readFile(
    join(exportDirectory(root, session.id, exportId), "export.json"),
    "utf8",
  );
  const candidate = parseExportCandidate(JSON.parse(raw));
  if (candidate.sessionId !== session.id || candidate.id !== exportId) {
    throw new Error("Export identity does not match its storage path.");
  }
  return candidate;
}

export async function getExportCandidatePayload(
  sessionId: string,
  exportId: string,
  root = workspaceRoot,
): Promise<ExportCandidatePayload> {
  const candidate = await getExportCandidate(sessionId, exportId, root);
  const directory = exportDirectory(root, sessionId, exportId);
  const [spriteSheetPngBytes, metadataBytes, provenanceBytes] =
    await Promise.all([
      readFile(join(directory, EXPORT_SHEET_FILE)),
      readFile(join(directory, EXPORT_METADATA_FILE)),
      readFile(join(directory, EXPORT_PROVENANCE_FILE)),
    ]);
  const sheet = PNG.sync.read(spriteSheetPngBytes, { checkCRC: true });
  if (
    spriteSheetPngBytes.byteLength !==
      candidate.package.spriteSheet.byteLength ||
    sha256(spriteSheetPngBytes) !== candidate.package.spriteSheet.sha256 ||
    sheet.width !== candidate.package.spriteSheet.width ||
    sheet.height !== candidate.package.spriteSheet.height
  ) {
    throw new Error(
      "Export sprite sheet no longer matches immutable provenance.",
    );
  }
  if (
    metadataBytes.byteLength !== candidate.package.metadata.byteLength ||
    sha256(metadataBytes) !== candidate.package.metadata.sha256
  ) {
    throw new Error("Export metadata no longer matches immutable provenance.");
  }
  if (
    provenanceBytes.byteLength !== candidate.package.provenance.byteLength ||
    sha256(provenanceBytes) !== candidate.package.provenance.sha256
  ) {
    throw new Error(
      "Export provenance no longer matches immutable provenance.",
    );
  }
  const metadata = parseExportMetadata(
    JSON.parse(metadataBytes.toString("utf8")),
  );
  const provenance = parseExportProvenance(
    JSON.parse(provenanceBytes.toString("utf8")),
  );
  if (
    provenance.exportId !== candidate.id ||
    provenance.sessionId !== candidate.sessionId ||
    JSON.stringify(provenance.approvedWorldTest) !==
      JSON.stringify(candidate.approvedWorldTest) ||
    JSON.stringify(provenance.sourceWalkCycle) !==
      JSON.stringify(candidate.sourceWalkCycle) ||
    JSON.stringify(provenance.preparation) !==
      JSON.stringify(candidate.preparation) ||
    JSON.stringify(provenance.publishing) !==
      JSON.stringify(candidate.publishing)
  ) {
    throw new Error("Export provenance document does not match its receipt.");
  }
  return { candidate, spriteSheetPngBytes, metadata, provenance };
}

export async function createExportCandidate(
  sessionId: string,
  sourceWorldTestId: string,
  options: CreateExportCandidateOptions = {},
): Promise<ExportCandidate> {
  const root = options.root ?? workspaceRoot;
  const session = await getSession(sessionId, root);
  const worldTest = await getWorldTestCandidatePayload(
    session.id,
    sourceWorldTestId,
    root,
  );
  const walkCycle = await getWalkCycleCandidatePayload(
    session.id,
    worldTest.candidate.sourceWalkCycle.walkCycleId,
    root,
  );
  const worldTestDocumentBytes = await readFile(
    sourceWorldTestDocumentPath(
      root,
      session.id,
      worldTest.candidate.id,
    ),
  );
  const existing = await listExportCandidates(session.id, root);
  const revision =
    options.revision ??
    existing.reduce(
      (highest, candidate) => Math.max(highest, candidate.revision),
      0,
    ) +
      1;
  const timestamp = options.timestamp ?? new Date().toISOString();
  const id = createExportCandidateId(
    revision,
    timestamp,
    options.idSuffix ?? randomUUID().slice(0, 8),
  );
  const approvedWorldTest = {
    worldTestId: worldTest.candidate.id,
    documentSha256: sha256(worldTestDocumentBytes),
    previewSources: worldTestPreviewReceipt(worldTest.candidate),
    approvedBy: "user" as const,
    approvedAt: timestamp,
  };
  const sourceWalkCycle = {
    walkCycleId: walkCycle.candidate.id,
    frameSources: walkCycle.candidate.frames.map((frame) => ({
      direction: frame.direction,
      frameIndex: frame.frameIndex,
      sha256: frame.sha256,
      byteLength: frame.byteLength,
    })),
  };
  const spriteSheetPngBytes = renderSpriteSheet(walkCycle);
  const metadata = buildMetadata(
    { name: session.brief.name, kind: session.brief.kind },
    walkCycle,
  );
  const provenance = buildProvenance(
    id,
    session.id,
    approvedWorldTest,
    sourceWalkCycle,
  );
  const metadataBytes = jsonBytes(metadata);
  const provenanceBytes = jsonBytes(provenance);
  const candidate = parseExportCandidate({
    schemaVersion: 1,
    id,
    revision,
    sessionId: session.id,
    stage: "export",
    contractId: session.contractId,
    approvedWorldTest,
    sourceWalkCycle,
    package: {
      spriteSheet: {
        sourceFile: EXPORT_SHEET_FILE,
        sha256: sha256(spriteSheetPngBytes),
        byteLength: spriteSheetPngBytes.byteLength,
        width: EXPORT_SHEET_WIDTH,
        height: EXPORT_SHEET_HEIGHT,
        cellWidth: TILEFORGE_ACTOR_CONTRACT.frame.width,
        cellHeight: TILEFORGE_ACTOR_CONTRACT.frame.height,
        layout: EXPORT_SHEET_LAYOUT,
      },
      metadata: {
        sourceFile: EXPORT_METADATA_FILE,
        sha256: sha256(metadataBytes),
        byteLength: metadataBytes.byteLength,
      },
      provenance: {
        sourceFile: EXPORT_PROVENANCE_FILE,
        sha256: sha256(provenanceBytes),
        byteLength: provenanceBytes.byteLength,
      },
    },
    createdAt: timestamp,
    preparation: provenance.preparation,
    status: "draft",
    publishing: provenance.publishing,
  });

  const candidatesRoot = exportRoot(root, session.id);
  await mkdir(candidatesRoot, { recursive: true });
  const finalDirectory = exportDirectory(root, session.id, candidate.id);
  const temporaryDirectory = join(
    candidatesRoot,
    `.${candidate.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });
  try {
    await writeFile(
      join(temporaryDirectory, EXPORT_SHEET_FILE),
      spriteSheetPngBytes,
      { flag: "wx" },
    );
    await writeFile(
      join(temporaryDirectory, EXPORT_METADATA_FILE),
      metadataBytes,
      { flag: "wx" },
    );
    await writeFile(
      join(temporaryDirectory, EXPORT_PROVENANCE_FILE),
      provenanceBytes,
      { flag: "wx" },
    );
    await writeFile(
      join(temporaryDirectory, "export.json"),
      jsonBytes(candidate),
      { flag: "wx" },
    );
    await rename(temporaryDirectory, finalDirectory);
    return candidate;
  } catch (error) {
    await rm(temporaryDirectory, { recursive: true, force: true });
    throw error;
  }
}

export async function validateExportCandidate(
  sessionId: string,
  exportId: string,
  root = workspaceRoot,
): Promise<ExportValidationReport> {
  const session = await getSession(sessionId, root);
  const payload = await getExportCandidatePayload(
    session.id,
    exportId,
    root,
  );
  const worldTest = await getWorldTestCandidatePayload(
    session.id,
    payload.candidate.approvedWorldTest.worldTestId,
    root,
  );
  const worldTestDocumentBytes = await readFile(
    sourceWorldTestDocumentPath(
      root,
      session.id,
      worldTest.candidate.id,
    ),
  );
  assertWorldTestReceipt(
    payload.candidate,
    worldTest,
    worldTestDocumentBytes,
  );
  const walkCycle = await getWalkCycleCandidatePayload(
    session.id,
    payload.candidate.sourceWalkCycle.walkCycleId,
    root,
  );
  assertSourceWalkCycle(payload.candidate, worldTest, walkCycle);
  const expectedSheet = PNG.sync.read(
    Buffer.from(renderSpriteSheet(walkCycle)),
    {
      checkCRC: true,
    },
  );
  const actualSheet = PNG.sync.read(
    Buffer.from(payload.spriteSheetPngBytes),
    { checkCRC: true },
  );
  if (!actualSheet.data.equals(expectedSheet.data)) {
    throw new Error(
      "Export sprite sheet pixels no longer match the source Walk Cycle.",
    );
  }
  const expectedMetadata = buildMetadata(
    { name: session.brief.name, kind: session.brief.kind },
    walkCycle,
  );
  if (JSON.stringify(payload.metadata) !== JSON.stringify(expectedMetadata)) {
    throw new Error("Export metadata no longer describes the source frames.");
  }
  const expectedProvenance = buildProvenance(
    payload.candidate.id,
    session.id,
    payload.candidate.approvedWorldTest,
    payload.candidate.sourceWalkCycle,
  );
  if (
    JSON.stringify(payload.provenance) !== JSON.stringify(expectedProvenance)
  ) {
    throw new Error("Export provenance no longer matches its source receipts.");
  }

  const messages = [
    "Approved World Test document and all sixteen previews are SHA-256 bound.",
    "All sixteen source Walk Cycle frames match the approved World Test receipt.",
    "The immutable 128 x 128 sprite sheet matches its SHA-256 receipt.",
    "Every sheet cell is pixel-identical to its source Walk Cycle frame.",
    "Metadata matches the actor contract, layout, timing, anchor, and source frames.",
    "Provenance matches the exact World Test approval and Walk Cycle receipts.",
    "Publishing remains not approved and requires a separate user decision.",
  ];
  return parseExportValidationReport({
    schemaVersion: 1,
    validatorId: EXPORT_VALIDATOR_ID,
    exportId: payload.candidate.id,
    contractId: payload.candidate.contractId,
    checks: EXPORT_VALIDATION_CHECKS.map((id, index) => ({
      id,
      status: "pass",
      message: messages[index],
    })),
    summary: {
      pass: EXPORT_VALIDATION_CHECKS.length,
      fail: 0,
      notAssessed: 0,
    },
    publishing: payload.candidate.publishing,
  });
}
