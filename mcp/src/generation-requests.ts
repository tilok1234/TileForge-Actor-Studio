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
  createConceptGenerationRequestId,
  GENERATION_REQUEST_CANDIDATE_COUNT_DEFAULT,
  GENERATION_REQUEST_ID_MAX_LENGTH,
  parseConceptGenerationRequest,
  type ConceptGenerationRequest,
} from "../../src/lib/studio/generation-request.js";
import { compileActorPrompt } from "../../src/lib/studio/prompt.js";
import { getSession, workspaceRoot } from "./storage.js";

interface CreateConceptGenerationRequestOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
  revision?: number;
}

function assertGenerationRequestId(requestId: string): string {
  if (
    requestId.length > GENERATION_REQUEST_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(requestId)
  ) {
    throw new Error("Invalid generation request id.");
  }
  return requestId;
}

function requestsRoot(root: string, sessionId: string): string {
  return join(root, "sessions", sessionId, "generation-requests");
}

function requestDirectory(
  root: string,
  sessionId: string,
  requestId: string,
): string {
  return join(
    requestsRoot(root, sessionId),
    assertGenerationRequestId(requestId),
  );
}

export async function listConceptGenerationRequests(
  sessionId: string,
  root = workspaceRoot,
): Promise<ConceptGenerationRequest[]> {
  const session = await getSession(sessionId, root);
  const rootDirectory = requestsRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const entries = await readdir(rootDirectory, { withFileTypes: true });
  const requests = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getConceptGenerationRequest(
            session.id,
            entry.name,
            root,
          );
        } catch {
          return null;
        }
      }),
  );

  return requests
    .filter(
      (request): request is ConceptGenerationRequest => request !== null,
    )
    .sort((left, right) => right.revision - left.revision);
}

export async function getConceptGenerationRequest(
  sessionId: string,
  requestId: string,
  root = workspaceRoot,
): Promise<ConceptGenerationRequest> {
  const session = await getSession(sessionId, root);
  const raw = await readFile(
    join(requestDirectory(root, session.id, requestId), "request.json"),
    "utf8",
  );
  const request = parseConceptGenerationRequest(JSON.parse(raw));
  if (request.sessionId !== session.id || request.id !== requestId) {
    throw new Error("Generation request identity does not match its storage path.");
  }
  return request;
}

export async function createConceptGenerationRequest(
  sessionId: string,
  requestedCandidates = GENERATION_REQUEST_CANDIDATE_COUNT_DEFAULT,
  options: CreateConceptGenerationRequestOptions = {},
): Promise<ConceptGenerationRequest> {
  const root = options.root ?? workspaceRoot;
  const session = await getSession(sessionId, root);
  const existingRequests = await listConceptGenerationRequests(session.id, root);
  const revision =
    options.revision ??
    existingRequests.reduce(
      (highest, request) => Math.max(highest, request.revision),
      0,
    ) + 1;
  const timestamp = options.timestamp ?? new Date().toISOString();
  const request = parseConceptGenerationRequest({
    schemaVersion: 1,
    id: createConceptGenerationRequestId(
      revision,
      timestamp,
      options.idSuffix ?? randomUUID().slice(0, 8),
    ),
    revision,
    sessionId: session.id,
    stage: "concept",
    contractId: session.contractId,
    createdAt: timestamp,
    prompt: compileActorPrompt(session.brief),
    requestedCandidates,
    execution: {
      mode: "connected-client-native-image-generation",
      additionalPaidServices: "forbidden",
      apiCredentials: "not-used",
    },
    output: {
      artifact: "concept-candidate",
      direction: "down",
      width: 32,
      height: 32,
      mimeType: "image/png",
      importTool: "import_concept_candidate",
    },
    authority: {
      agentsMayGenerate: true,
      agentsMayImport: true,
      agentsMayApprove: false,
      approvalOwner: "user",
    },
    lifecycle: "immutable-request",
  });

  const rootDirectory = requestsRoot(root, session.id);
  await mkdir(rootDirectory, { recursive: true });
  const finalDirectory = requestDirectory(root, session.id, request.id);
  const temporaryDirectory = join(
    rootDirectory,
    `.${request.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });

  try {
    await writeFile(
      join(temporaryDirectory, "request.json"),
      `${JSON.stringify(request, null, 2)}\n`,
      { encoding: "utf8", flag: "wx" },
    );
    await rename(temporaryDirectory, finalDirectory);
    return request;
  } catch (error) {
    await rm(temporaryDirectory, { recursive: true, force: true });
    throw error;
  }
}
