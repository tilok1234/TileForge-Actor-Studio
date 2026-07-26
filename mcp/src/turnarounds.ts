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
  createTurnaroundCandidateId,
  parseTurnaroundCandidate,
  TURNAROUND_DIRECTIONS,
  TURNAROUND_ID_MAX_LENGTH,
  type TurnaroundCandidate,
  type TurnaroundDirection,
} from "../../src/lib/studio/turnaround.js";
import {
  parseTurnaroundValidationReport,
  TURNAROUND_VALIDATOR_ID,
  type TurnaroundValidationReport,
} from "../../src/lib/studio/turnaround-validation.js";
import { validatePngStructuralEvidence } from "../../src/lib/studio/validate-candidate.js";
import {
  getConceptCandidatePayload,
  validateConceptPng,
} from "./candidates.js";
import { getSession, workspaceRoot } from "./storage.js";

export type TurnaroundPngs = Record<TurnaroundDirection, Uint8Array>;

export interface TurnaroundCandidatePayload {
  candidate: TurnaroundCandidate;
  pngBytes: TurnaroundPngs;
}

interface CreateTurnaroundCandidateOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
  revision?: number;
}

function assertTurnaroundId(turnaroundId: string): string {
  if (
    turnaroundId.length > TURNAROUND_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(turnaroundId)
  ) {
    throw new Error("Invalid turnaround id.");
  }
  return turnaroundId;
}

function turnaroundRoot(root: string, sessionId: string): string {
  return join(root, "sessions", sessionId, "turnarounds");
}

function turnaroundDirectory(
  root: string,
  sessionId: string,
  turnaroundId: string,
): string {
  return join(
    turnaroundRoot(root, sessionId),
    assertTurnaroundId(turnaroundId),
  );
}

export async function listTurnaroundCandidates(
  sessionId: string,
  root = workspaceRoot,
): Promise<TurnaroundCandidate[]> {
  const session = await getSession(sessionId, root);
  const rootDirectory = turnaroundRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const entries = await readdir(rootDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getTurnaroundCandidate(session.id, entry.name, root);
        } catch {
          return null;
        }
      }),
  );

  return candidates
    .filter((candidate): candidate is TurnaroundCandidate => candidate !== null)
    .sort((left, right) => right.revision - left.revision);
}

export async function getTurnaroundCandidate(
  sessionId: string,
  turnaroundId: string,
  root = workspaceRoot,
): Promise<TurnaroundCandidate> {
  const session = await getSession(sessionId, root);
  const raw = await readFile(
    join(
      turnaroundDirectory(root, session.id, turnaroundId),
      "turnaround.json",
    ),
    "utf8",
  );
  const candidate = parseTurnaroundCandidate(JSON.parse(raw));
  if (candidate.sessionId !== session.id || candidate.id !== turnaroundId) {
    throw new Error("Turnaround identity does not match its storage path.");
  }
  return candidate;
}

export async function getTurnaroundCandidatePayload(
  sessionId: string,
  turnaroundId: string,
  root = workspaceRoot,
): Promise<TurnaroundCandidatePayload> {
  const candidate = await getTurnaroundCandidate(
    sessionId,
    turnaroundId,
    root,
  );
  const directory = turnaroundDirectory(root, sessionId, turnaroundId);
  const entries = await Promise.all(
    candidate.directions.map(async (source) => {
      const bytes = await readFile(join(directory, source.sourceFile));
      const evidence = validateConceptPng(bytes);
      if (
        evidence.sha256 !== source.sha256 ||
        bytes.byteLength !== source.byteLength
      ) {
        throw new Error(
          `${source.direction} source bytes no longer match immutable provenance.`,
        );
      }
      return [source.direction, bytes] as const;
    }),
  );
  return {
    candidate,
    pngBytes: Object.fromEntries(entries) as unknown as TurnaroundPngs,
  };
}

export async function createTurnaroundCandidate(
  sessionId: string,
  sourceConceptId: string,
  pngBytes: TurnaroundPngs,
  provenanceInput: CandidateProvenance,
  options: CreateTurnaroundCandidateOptions = {},
): Promise<TurnaroundCandidate> {
  const root = options.root ?? workspaceRoot;
  const session = await getSession(sessionId, root);
  const sourceConcept = await getConceptCandidatePayload(
    session.id,
    sourceConceptId,
    root,
  );
  const provenance = parseCandidateProvenance(provenanceInput);
  const evidence = Object.fromEntries(
    TURNAROUND_DIRECTIONS.map((direction) => [
      direction,
      validateConceptPng(pngBytes[direction]),
    ]),
  ) as Record<TurnaroundDirection, ReturnType<typeof validateConceptPng>>;
  if (
    evidence.down.sha256 !== sourceConcept.candidate.sha256 ||
    pngBytes.down.byteLength !== sourceConcept.pngBytes.byteLength
  ) {
    throw new Error(
      "Down view must preserve the exact user-selected Concept PNG bytes.",
    );
  }

  const existingCandidates = await listTurnaroundCandidates(session.id, root);
  const revision =
    options.revision ??
    existingCandidates.reduce(
      (highest, candidate) => Math.max(highest, candidate.revision),
      0,
    ) + 1;
  const timestamp = options.timestamp ?? new Date().toISOString();
  const candidate = parseTurnaroundCandidate({
    schemaVersion: 1,
    id: createTurnaroundCandidateId(
      revision,
      timestamp,
      options.idSuffix ?? randomUUID().slice(0, 8),
    ),
    revision,
    sessionId: session.id,
    stage: "turnaround",
    contractId: session.contractId,
    sourceSelection: {
      candidateId: sourceConcept.candidate.id,
      candidateSha256: sourceConcept.candidate.sha256,
      selectedBy: "user",
      selectedAt: timestamp,
    },
    directions: TURNAROUND_DIRECTIONS.map((direction) => ({
      direction,
      sourceFile: `${direction}.png`,
      sha256: evidence[direction].sha256,
      byteLength: pngBytes[direction].byteLength,
      width: evidence[direction].width,
      height: evidence[direction].height,
    })),
    createdAt: timestamp,
    provenance,
    reviewStatus: "unreviewed",
    identityJudgment: {
      status: "not_assessed",
      authority: "user",
      message:
        "Only the user can accept identity consistency across turnaround views.",
    },
  });

  const candidatesRoot = turnaroundRoot(root, session.id);
  await mkdir(candidatesRoot, { recursive: true });
  const finalDirectory = turnaroundDirectory(root, session.id, candidate.id);
  const temporaryDirectory = join(
    candidatesRoot,
    `.${candidate.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });

  try {
    for (const direction of TURNAROUND_DIRECTIONS) {
      await writeFile(
        join(temporaryDirectory, `${direction}.png`),
        pngBytes[direction],
        { flag: "wx" },
      );
    }
    await writeFile(
      join(temporaryDirectory, "turnaround.json"),
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

export async function validateTurnaroundCandidate(
  sessionId: string,
  turnaroundId: string,
  root = workspaceRoot,
): Promise<TurnaroundValidationReport> {
  const payload = await getTurnaroundCandidatePayload(
    sessionId,
    turnaroundId,
    root,
  );
  const directions = payload.candidate.directions.map((source) => ({
    direction: source.direction,
    report: validatePngStructuralEvidence(
      {
        artifactId: payload.candidate.id,
        sha256: source.sha256,
        byteLength: source.byteLength,
        contractId: payload.candidate.contractId,
      },
      payload.pngBytes[source.direction],
    ),
  }));
  const summary = directions.reduce(
    (totals, direction) => ({
      pass: totals.pass + direction.report.summary.pass,
      fail: totals.fail + direction.report.summary.fail,
      notAssessed:
        totals.notAssessed + direction.report.summary.notAssessed,
    }),
    { pass: 0, fail: 0, notAssessed: 0 },
  );
  return parseTurnaroundValidationReport({
    schemaVersion: 1,
    validatorId: TURNAROUND_VALIDATOR_ID,
    turnaroundId: payload.candidate.id,
    contractId: payload.candidate.contractId,
    directions,
    summary,
    identityJudgment: payload.candidate.identityJudgment,
  });
}
