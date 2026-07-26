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
import {
  CANDIDATE_ID_MAX_LENGTH,
  CONCEPT_PNG_MAX_BYTES,
  createConceptCandidateId,
  parseCandidateProvenance,
  parseConceptCandidate,
  type CandidateProvenance,
  type ConceptCandidate,
} from "../../src/lib/studio/candidate.js";
import { TILEFORGE_ACTOR_CONTRACT } from "../../src/lib/studio/contract.js";
import { getSession, workspaceRoot } from "./storage.js";

export interface CandidateImageEvidence {
  width: 32;
  height: 32;
  sha256: string;
}

interface CreateConceptCandidateOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
  revision?: number;
}

export interface ConceptCandidatePayload {
  candidate: ConceptCandidate;
  pngBytes: Uint8Array;
}

function assertCandidateId(candidateId: string): string {
  if (
    candidateId.length > CANDIDATE_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(candidateId)
  ) {
    throw new Error("Invalid candidate id.");
  }
  return candidateId;
}

function candidateRoot(root: string, sessionId: string): string {
  return join(root, "sessions", sessionId, "candidates");
}

function candidateDirectory(
  root: string,
  sessionId: string,
  candidateId: string,
): string {
  return join(candidateRoot(root, sessionId), assertCandidateId(candidateId));
}

export function validateConceptPng(
  pngBytes: Uint8Array,
): CandidateImageEvidence {
  if (pngBytes.byteLength === 0) {
    throw new Error("PNG file is empty.");
  }
  if (pngBytes.byteLength > CONCEPT_PNG_MAX_BYTES) {
    throw new Error("PNG file exceeds the 1 MiB intake limit.");
  }

  let decoded: PNG;
  try {
    decoded = PNG.sync.read(Buffer.from(pngBytes), { checkCRC: true });
  } catch {
    throw new Error("File is not a valid PNG.");
  }

  if (
    decoded.width !== TILEFORGE_ACTOR_CONTRACT.frame.width ||
    decoded.height !== TILEFORGE_ACTOR_CONTRACT.frame.height
  ) {
    throw new Error("Concept PNG must be exactly 32 x 32 pixels.");
  }

  let hasTransparency = false;
  for (let index = 3; index < decoded.data.length; index += 4) {
    if (decoded.data[index] < 255) {
      hasTransparency = true;
      break;
    }
  }
  if (!hasTransparency) {
    throw new Error("Concept PNG must contain an alpha channel with transparency.");
  }

  return {
    width: 32,
    height: 32,
    sha256: createHash("sha256").update(pngBytes).digest("hex"),
  };
}

export async function listConceptCandidates(
  sessionId: string,
  root = workspaceRoot,
): Promise<ConceptCandidate[]> {
  const session = await getSession(sessionId, root);
  const rootDirectory = candidateRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const entries = await readdir(rootDirectory, { withFileTypes: true });
  const candidates = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getConceptCandidate(session.id, entry.name, root);
        } catch {
          return null;
        }
      }),
  );

  return candidates
    .filter((candidate): candidate is ConceptCandidate => candidate !== null)
    .sort((left, right) => right.revision - left.revision);
}

export async function getConceptCandidate(
  sessionId: string,
  candidateId: string,
  root = workspaceRoot,
): Promise<ConceptCandidate> {
  const session = await getSession(sessionId, root);
  const raw = await readFile(
    join(candidateDirectory(root, session.id, candidateId), "candidate.json"),
    "utf8",
  );
  const candidate = parseConceptCandidate(JSON.parse(raw));
  if (candidate.sessionId !== session.id || candidate.id !== candidateId) {
    throw new Error("Candidate identity does not match its storage path.");
  }
  return candidate;
}

export async function getConceptCandidatePayload(
  sessionId: string,
  candidateId: string,
  root = workspaceRoot,
): Promise<ConceptCandidatePayload> {
  const candidate = await getConceptCandidate(sessionId, candidateId, root);
  const pngBytes = await readFile(
    join(candidateDirectory(root, sessionId, candidateId), candidate.sourceFile),
  );
  const sha256 = createHash("sha256").update(pngBytes).digest("hex");
  if (sha256 !== candidate.sha256 || pngBytes.byteLength !== candidate.byteLength) {
    throw new Error("Candidate source bytes no longer match immutable provenance.");
  }
  return { candidate, pngBytes };
}

export async function createConceptCandidate(
  sessionId: string,
  pngBytes: Uint8Array,
  provenanceInput: CandidateProvenance,
  options: CreateConceptCandidateOptions = {},
): Promise<ConceptCandidate> {
  const root = options.root ?? workspaceRoot;
  const session = await getSession(sessionId, root);
  const provenance = parseCandidateProvenance(provenanceInput);
  const evidence = validateConceptPng(pngBytes);
  const existingCandidates = await listConceptCandidates(session.id, root);
  const revision =
    options.revision ??
    existingCandidates.reduce(
      (highest, candidate) => Math.max(highest, candidate.revision),
      0,
    ) + 1;
  const timestamp = options.timestamp ?? new Date().toISOString();
  const candidate: ConceptCandidate = parseConceptCandidate({
    schemaVersion: 1,
    id: createConceptCandidateId(
      revision,
      timestamp,
      options.idSuffix ?? randomUUID().slice(0, 8),
    ),
    revision,
    sessionId: session.id,
    stage: "concept",
    direction: "down",
    contractId: session.contractId,
    sourceFile: "source.png",
    mimeType: "image/png",
    sha256: evidence.sha256,
    byteLength: pngBytes.byteLength,
    width: evidence.width,
    height: evidence.height,
    createdAt: timestamp,
    provenance,
    intakeValidation: {
      fileType: "pass",
      dimensions: "pass",
      alphaChannel: "pass",
    },
    reviewStatus: "unreviewed",
  });

  const candidatesRoot = candidateRoot(root, session.id);
  await mkdir(candidatesRoot, { recursive: true });
  const finalDirectory = candidateDirectory(root, session.id, candidate.id);
  const temporaryDirectory = join(
    candidatesRoot,
    `.${candidate.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });

  try {
    await writeFile(join(temporaryDirectory, candidate.sourceFile), pngBytes, {
      flag: "wx",
    });
    await writeFile(
      join(temporaryDirectory, "candidate.json"),
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
