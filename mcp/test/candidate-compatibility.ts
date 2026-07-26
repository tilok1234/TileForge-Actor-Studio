import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import {
  mkdtemp,
  readFile,
  readdir,
  rm,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import {
  parseConceptCandidate,
  type ConceptCandidate,
} from "../../src/lib/studio/candidate.js";
import {
  createConceptCandidate,
  getConceptCandidatePayload,
} from "../src/candidates.js";
import { createActorStudioServer } from "../src/server.js";
import { createSession } from "../src/storage.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "concept-candidate-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-m02-"));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function conceptPng(
  width = 32,
  height = 32,
  transparent = true,
): Uint8Array {
  const png = new PNG({ width, height });
  for (let index = 0; index < png.data.length; index += 4) {
    png.data[index] = 42;
    png.data[index + 1] = 74;
    png.data[index + 2] = 55;
    png.data[index + 3] = transparent ? 0 : 255;
  }
  const center = (Math.floor(height / 2) * width + Math.floor(width / 2)) * 4;
  png.data[center] = 104;
  png.data[center + 1] = 198;
  png.data[center + 2] = 178;
  png.data[center + 3] = 255;
  return PNG.sync.write(png);
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-candidate-compatibility",
  version: "0.1.0",
});

try {
  const session = await createSession(
    {
      name: "Mirelight Pilgrim",
      kind: "mob",
      description: "A quiet reed-cloaked marsh pilgrim.",
    },
    {
      root,
      timestamp: "2026-07-26T21:30:00.000Z",
      idSuffix: "m02test1",
    },
  );
  await Promise.all([server.connect(serverTransport), client.connect(clientTransport)]);

  for (const [label, bytes, mimeType] of [
    ["invalid dimensions", conceptPng(31, 32), "image/png"],
    ["missing transparency", conceptPng(32, 32, false), "image/png"],
    ["invalid file type", new TextEncoder().encode("not a png"), "image/png"],
  ] as const) {
    let failed = false;
    try {
      await createConceptCandidate(
        session.id,
        bytes,
        {
          source: "imported",
          originalFilename: `${label}.png`,
        },
        { root },
      );
    } catch {
      failed = true;
    }
    assert(failed, `${label} unexpectedly created a candidate.`);
    assert(mimeType === "image/png", "Test setup changed unexpectedly.");
  }
  assert(
    (await readdir(join(root, "sessions", session.id, "candidates"))).length === 0,
    "Rejected intake left a partial candidate directory.",
  );

  const sourceBytes = conceptPng();
  const imported = await client.callTool({
    name: "import_concept_candidate",
    arguments: {
      sessionId: session.id,
      pngBase64: Buffer.from(sourceBytes).toString("base64"),
      provenance: {
        source: "imported",
        originalFilename: "mirelight-pilgrim.png",
      },
    },
  });
  assert(imported.isError !== true, "MCP rejected a valid Concept PNG.");
  const firstCandidate = parseConceptCandidate(JSON.parse(textPayload(imported)));
  assert(
    firstCandidate.reviewStatus === "unreviewed",
    "Candidate creation implied visual approval.",
  );

  const listed = await client.callTool({
    name: "list_concept_candidates",
    arguments: { sessionId: session.id },
  });
  const listedPayload = JSON.parse(textPayload(listed)) as {
    candidates: ConceptCandidate[];
  };
  assert(
    listedPayload.candidates.length === 1 &&
      listedPayload.candidates[0]?.id === firstCandidate.id,
    "MCP list did not return the imported candidate.",
  );

  const fetched = await client.callTool({
    name: "get_concept_candidate",
    arguments: {
      sessionId: session.id,
      candidateId: firstCandidate.id,
    },
  });
  const fetchedPayload = JSON.parse(textPayload(fetched)) as {
    candidate: ConceptCandidate;
    pngBase64: string;
  };
  assert(
    Buffer.from(fetchedPayload.pngBase64, "base64").equals(Buffer.from(sourceBytes)),
    "MCP did not return the original immutable PNG bytes.",
  );

  const collisionIdentity = {
    root,
    timestamp: "2026-07-26T21:31:00.000Z",
    idSuffix: "same0001",
    revision: 2,
  };
  const secondCandidate = await createConceptCandidate(
    session.id,
    sourceBytes,
    {
      source: "generated",
      provider: "test-provider",
      model: "test-model",
    },
    collisionIdentity,
  );
  let collisionFailed = false;
  try {
    await createConceptCandidate(
      session.id,
      conceptPng(),
      {
        source: "generated",
        provider: "test-provider",
      },
      {
        ...collisionIdentity,
        temporarySuffix: "collision-cleanup",
      },
    );
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "Candidate identity collision overwrote original bytes.");
  const candidatesRoot = join(root, "sessions", session.id, "candidates");
  const candidateEntries = await readdir(candidatesRoot);
  assert(
    candidateEntries.length === 2 &&
      !candidateEntries.some((name) => name.startsWith(".")),
    "Failed candidate creation left a partial directory.",
  );
  const secondPayload = await getConceptCandidatePayload(
    session.id,
    secondCandidate.id,
    root,
  );
  assert(
    Buffer.from(secondPayload.pngBytes).equals(Buffer.from(sourceBytes)),
    "Collision changed the original candidate bytes.",
  );

  const fixture = parseConceptCandidate(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let approvalFieldRejected = false;
  try {
    parseConceptCandidate({ ...fixture, approved: true });
  } catch {
    approvalFieldRejected = true;
  }
  assert(
    approvalFieldRejected,
    "Candidate document accepted an agent-writable approval field.",
  );

  console.log(
    "Candidate compatibility passed (PNG intake, immutable bytes, provenance, MCP reads, failure cleanup).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
