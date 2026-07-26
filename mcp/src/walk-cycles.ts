import { randomUUID } from "node:crypto";
import {
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import {
  parseCandidateProvenance,
  type CandidateProvenance,
} from "../../src/lib/studio/candidate.js";
import {
  TURNAROUND_DIRECTIONS,
  type TurnaroundDirection,
} from "../../src/lib/studio/turnaround.js";
import {
  createWalkCycleCandidateId,
  parseWalkCycleCandidate,
  WALK_CYCLE_FRAMES_PER_DIRECTION,
  WALK_CYCLE_ID_MAX_LENGTH,
  type WalkCycleCandidate,
} from "../../src/lib/studio/walk-cycle.js";
import {
  parseWalkCycleValidationReport,
  WALK_CYCLE_VALIDATOR_ID,
  type WalkCycleValidationReport,
} from "../../src/lib/studio/walk-cycle-validation.js";
import { validatePngStructuralEvidence } from "../../src/lib/studio/validate-candidate.js";
import { validateConceptPng } from "./candidates.js";
import { getSession, workspaceRoot } from "./storage.js";
import { getTurnaroundCandidatePayload } from "./turnarounds.js";

export type WalkCyclePngs = Record<TurnaroundDirection, Uint8Array[]>;

export interface WalkCycleCandidatePayload {
  candidate: WalkCycleCandidate;
  pngBytes: WalkCyclePngs;
}

interface CreateWalkCycleCandidateOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
  revision?: number;
}

function assertWalkCycleId(walkCycleId: string): string {
  if (
    walkCycleId.length > WALK_CYCLE_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(walkCycleId)
  ) {
    throw new Error("Invalid Walk Cycle id.");
  }
  return walkCycleId;
}

function walkCycleRoot(root: string, sessionId: string): string {
  return join(root, "sessions", sessionId, "walk-cycles");
}

function walkCycleDirectory(
  root: string,
  sessionId: string,
  walkCycleId: string,
): string {
  return join(
    walkCycleRoot(root, sessionId),
    assertWalkCycleId(walkCycleId),
  );
}

function assertCanonicalFrameSet(pngBytes: WalkCyclePngs): void {
  for (const direction of TURNAROUND_DIRECTIONS) {
    if (
      !Array.isArray(pngBytes[direction]) ||
      pngBytes[direction].length !== WALK_CYCLE_FRAMES_PER_DIRECTION
    ) {
      throw new Error(
        `Walk Cycle ${direction} must contain exactly ${WALK_CYCLE_FRAMES_PER_DIRECTION} frames.`,
      );
    }
  }
}

export async function listWalkCycleCandidates(
  sessionId: string,
  root = workspaceRoot,
): Promise<WalkCycleCandidate[]> {
  const session = await getSession(sessionId, root);
  const rootDirectory = walkCycleRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const entries = await readdir(rootDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getWalkCycleCandidate(session.id, entry.name, root);
        } catch {
          return null;
        }
      }),
  );

  return candidates
    .filter((candidate): candidate is WalkCycleCandidate => candidate !== null)
    .sort((left, right) => right.revision - left.revision);
}

export async function getWalkCycleCandidate(
  sessionId: string,
  walkCycleId: string,
  root = workspaceRoot,
): Promise<WalkCycleCandidate> {
  const session = await getSession(sessionId, root);
  const raw = await readFile(
    join(
      walkCycleDirectory(root, session.id, walkCycleId),
      "walk-cycle.json",
    ),
    "utf8",
  );
  const candidate = parseWalkCycleCandidate(JSON.parse(raw));
  if (candidate.sessionId !== session.id || candidate.id !== walkCycleId) {
    throw new Error("Walk Cycle identity does not match its storage path.");
  }
  return candidate;
}

export async function getWalkCycleCandidatePayload(
  sessionId: string,
  walkCycleId: string,
  root = workspaceRoot,
): Promise<WalkCycleCandidatePayload> {
  const candidate = await getWalkCycleCandidate(
    sessionId,
    walkCycleId,
    root,
  );
  const directory = walkCycleDirectory(root, sessionId, walkCycleId);
  const pngBytes = Object.fromEntries(
    TURNAROUND_DIRECTIONS.map((direction) => [direction, []]),
  ) as unknown as WalkCyclePngs;

  for (const source of candidate.frames) {
    const bytes = await readFile(join(directory, source.sourceFile));
    const evidence = validateConceptPng(bytes);
    if (
      evidence.sha256 !== source.sha256 ||
      bytes.byteLength !== source.byteLength
    ) {
      throw new Error(
        `${source.direction} frame ${source.frameIndex} bytes no longer match immutable provenance.`,
      );
    }
    pngBytes[source.direction].push(bytes);
  }

  assertCanonicalFrameSet(pngBytes);
  return { candidate, pngBytes };
}

export async function createWalkCycleCandidate(
  sessionId: string,
  sourceTurnaroundId: string,
  pngBytes: WalkCyclePngs,
  provenanceInput: CandidateProvenance,
  options: CreateWalkCycleCandidateOptions = {},
): Promise<WalkCycleCandidate> {
  const root = options.root ?? workspaceRoot;
  const session = await getSession(sessionId, root);
  const sourceTurnaround = await getTurnaroundCandidatePayload(
    session.id,
    sourceTurnaroundId,
    root,
  );
  const provenance = parseCandidateProvenance(provenanceInput);
  assertCanonicalFrameSet(pngBytes);

  const evidence = Object.fromEntries(
    TURNAROUND_DIRECTIONS.map((direction) => [
      direction,
      pngBytes[direction].map((bytes) => validateConceptPng(bytes)),
    ]),
  ) as Record<
    TurnaroundDirection,
    ReturnType<typeof validateConceptPng>[]
  >;

  for (const source of sourceTurnaround.candidate.directions) {
    const firstFrame = pngBytes[source.direction][0];
    const firstEvidence = evidence[source.direction][0];
    if (
      !firstFrame ||
      !firstEvidence ||
      firstEvidence.sha256 !== source.sha256 ||
      firstFrame.byteLength !== source.byteLength
    ) {
      throw new Error(
        `Frame 0 for ${source.direction} must preserve the exact user-accepted Turnaround PNG bytes.`,
      );
    }
  }

  const existingCandidates = await listWalkCycleCandidates(session.id, root);
  const revision =
    options.revision ??
    existingCandidates.reduce(
      (highest, candidate) => Math.max(highest, candidate.revision),
      0,
    ) + 1;
  const timestamp = options.timestamp ?? new Date().toISOString();
  const candidate = parseWalkCycleCandidate({
    schemaVersion: 1,
    id: createWalkCycleCandidateId(
      revision,
      timestamp,
      options.idSuffix ?? randomUUID().slice(0, 8),
    ),
    revision,
    sessionId: session.id,
    stage: "animate",
    contractId: session.contractId,
    sourceTurnaround: {
      turnaroundId: sourceTurnaround.candidate.id,
      directionSources: sourceTurnaround.candidate.directions.map(
        (source) => ({
          direction: source.direction,
          sha256: source.sha256,
          byteLength: source.byteLength,
        }),
      ),
      acceptedBy: "user",
      acceptedAt: timestamp,
    },
    clip: "walk",
    framesPerDirection: WALK_CYCLE_FRAMES_PER_DIRECTION,
    frameDurationMs: 300,
    frames: TURNAROUND_DIRECTIONS.flatMap((direction) =>
      evidence[direction].map((frameEvidence, frameIndex) => ({
        direction,
        frameIndex,
        sourceFile: `${direction}-${frameIndex}.png`,
        sha256: frameEvidence.sha256,
        byteLength: pngBytes[direction][frameIndex]?.byteLength,
        width: frameEvidence.width,
        height: frameEvidence.height,
      })),
    ),
    createdAt: timestamp,
    provenance,
    reviewStatus: "unreviewed",
    motionJudgment: {
      status: "not_assessed",
      authority: "user",
      message:
        "Only the user can accept Walk Cycle motion and readability.",
    },
  });

  const candidatesRoot = walkCycleRoot(root, session.id);
  await mkdir(candidatesRoot, { recursive: true });
  const finalDirectory = walkCycleDirectory(root, session.id, candidate.id);
  const temporaryDirectory = join(
    candidatesRoot,
    `.${candidate.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });

  try {
    for (const source of candidate.frames) {
      const bytes = pngBytes[source.direction][source.frameIndex];
      if (!bytes) {
        throw new Error("Walk Cycle frame bytes are incomplete.");
      }
      await writeFile(join(temporaryDirectory, source.sourceFile), bytes, {
        flag: "wx",
      });
    }
    await writeFile(
      join(temporaryDirectory, "walk-cycle.json"),
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

export async function validateWalkCycleCandidate(
  sessionId: string,
  walkCycleId: string,
  root = workspaceRoot,
): Promise<WalkCycleValidationReport> {
  const payload = await getWalkCycleCandidatePayload(
    sessionId,
    walkCycleId,
    root,
  );
  const frames = payload.candidate.frames.map((source) => ({
    direction: source.direction,
    frameIndex: source.frameIndex,
    report: validatePngStructuralEvidence(
      {
        artifactId: payload.candidate.id,
        sha256: source.sha256,
        byteLength: source.byteLength,
        contractId: payload.candidate.contractId,
      },
      payload.pngBytes[source.direction][source.frameIndex]!,
    ),
  }));
  const summary = frames.reduce(
    (totals, frame) => ({
      pass: totals.pass + frame.report.summary.pass,
      fail: totals.fail + frame.report.summary.fail,
      notAssessed:
        totals.notAssessed + frame.report.summary.notAssessed,
    }),
    { pass: 0, fail: 0, notAssessed: 0 },
  );
  return parseWalkCycleValidationReport({
    schemaVersion: 1,
    validatorId: WALK_CYCLE_VALIDATOR_ID,
    walkCycleId: payload.candidate.id,
    contractId: payload.candidate.contractId,
    frames,
    summary,
    motionJudgment: payload.candidate.motionJudgment,
  });
}
