import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseConceptGenerationRequest } from "../../src/lib/studio/generation-request.js";
import { createActorStudioServer } from "../src/server.js";
import {
  createConceptGenerationRequest,
  getConceptGenerationRequest,
} from "../src/generation-requests.js";
import { createSession } from "../src/storage.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "concept-generation-request-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-generation-request-"));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-generation-request-compatibility",
  version: "0.1.0",
});

try {
  await Promise.all([server.connect(serverTransport), client.connect(clientTransport)]);

  const session = await createSession(
    {
      name: "Request Keeper",
      kind: "npc",
      description: "Carries one local lantern and no paid API credentials.",
    },
    {
      root,
      timestamp: "2026-07-27T12:00:00.000Z",
      idSuffix: "request1",
    },
  );

  const created = await createConceptGenerationRequest(session.id, 3, {
    root,
    timestamp: "2026-07-27T12:01:00.000Z",
    idSuffix: "request1",
  });
  assert(
    created.execution.additionalPaidServices === "forbidden" &&
      created.execution.apiCredentials === "not-used" &&
      created.authority.agentsMayApprove === false,
    "Generation request weakened the cost or approval boundary.",
  );
  assert(
    created.prompt.includes("Only the user may approve final art"),
    "Generation request omitted the human approval boundary.",
  );

  const reread = await getConceptGenerationRequest(session.id, created.id, root);
  assert(reread.id === created.id, "Durable generation request could not be reread.");

  let collisionFailed = false;
  try {
    await createConceptGenerationRequest(session.id, 3, {
      root,
      timestamp: "2026-07-27T12:01:00.000Z",
      idSuffix: "request1",
      revision: 1,
      temporarySuffix: "collision-cleanup",
    });
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "Generation request collision overwrote an existing request.");
  const requestEntries = await readdir(
    join(root, "sessions", session.id, "generation-requests"),
  );
  assert(
    requestEntries.length === 1 &&
      !requestEntries.some((name) => name.startsWith(".")),
    "Failed generation request creation left partial storage.",
  );

  const invalidCount = await client.callTool({
    name: "create_concept_generation_request",
    arguments: { sessionId: session.id, requestedCandidates: 5 },
  });
  assert(
    invalidCount.isError === true,
    "MCP accepted a generation request outside the 1-4 candidate limit.",
  );
  assert(
    (await readdir(
      join(root, "sessions", session.id, "generation-requests"),
    )).length === 1,
    "Rejected MCP generation request left partial storage.",
  );

  const listed = await client.callTool({
    name: "list_concept_generation_requests",
    arguments: { sessionId: session.id },
  });
  const listedPayload = JSON.parse(textPayload(listed)) as {
    requests: unknown[];
  };
  assert(
    parseConceptGenerationRequest(listedPayload.requests[0]).id === created.id,
    "MCP did not list the desktop-compatible generation request.",
  );

  const fetched = await client.callTool({
    name: "get_concept_generation_request",
    arguments: { sessionId: session.id, requestId: created.id },
  });
  assert(
    parseConceptGenerationRequest(JSON.parse(textPayload(fetched))).id ===
      created.id,
    "MCP could not read one durable generation request.",
  );

  const fixture = parseConceptGenerationRequest(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let approvalRejected = false;
  try {
    parseConceptGenerationRequest({
      ...fixture,
      authority: { ...fixture.authority, agentsMayApprove: true },
    });
  } catch {
    approvalRejected = true;
  }
  assert(approvalRejected, "Generation request accepted agent approval authority.");

  const fixtureSessionDirectory = join(
    root,
    "sessions",
    fixture.sessionId,
  );
  await mkdir(
    join(
      fixtureSessionDirectory,
      "generation-requests",
      fixture.id,
    ),
    { recursive: true },
  );
  await cp(
    join(repositoryRoot, "tests", "fixtures", "session-v1.json"),
    join(fixtureSessionDirectory, "session.json"),
  );
  await cp(
    fixturePath,
    join(
      fixtureSessionDirectory,
      "generation-requests",
      fixture.id,
      "request.json",
    ),
  );
  const fixtureRead = await client.callTool({
    name: "get_concept_generation_request",
    arguments: { sessionId: fixture.sessionId, requestId: fixture.id },
  });
  assert(
    parseConceptGenerationRequest(JSON.parse(textPayload(fixtureRead))).id ===
      fixture.id,
    "MCP could not read the shared generation request fixture.",
  );

  console.log(
    "Generation request compatibility passed (subscription-only boundary, immutable identity, MCP reads, collision cleanup).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
