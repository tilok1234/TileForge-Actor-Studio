import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { createConceptCandidate } from "../src/candidates.js";
import { createActorStudioServer } from "../src/server.js";
import { createSession } from "../src/storage.js";
import { createTurnaroundCandidate } from "../src/turnarounds.js";
import {
  createWalkCycleCandidate,
  type WalkCyclePngs,
} from "../src/walk-cycles.js";
import {
  createWorldTestCandidate,
  getWorldTestCandidatePayload,
} from "../src/world-tests.js";
import {
  type TurnaroundDirection,
} from "../../src/lib/studio/turnaround.js";
import {
  parseWorldTestCandidate,
  type WorldTestCandidate,
} from "../../src/lib/studio/world-test.js";
import { parseWorldTestValidationReport } from "../../src/lib/studio/world-test-validation.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "world-test-candidate-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-m05-world-test-"));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}
function actorPng(offset = 0): Uint8Array {
  const png = new PNG({ width: 32, height: 32 });
  png.data.fill(0);
  for (let y = 5; y <= 28; y += 1) {
    for (let x = 10; x <= 21; x += 1) {
      const index = (y * 32 + x) * 4;
      png.data[index] = 6 + offset;
      png.data[index + 1] = 10 + offset;
      png.data[index + 2] = 8 + offset;
      png.data[index + 3] = 255;
    }
  }
  return PNG.sync.write(png);
}

function frameSet(
  views: Record<TurnaroundDirection, Uint8Array>,
): WalkCyclePngs {
  return Object.fromEntries(
    Object.entries(views).map(([direction, first], directionIndex) => [
      direction,
      [
        first,
        actorPng(1 + directionIndex),
        actorPng(2 + directionIndex),
        actorPng(3 + directionIndex),
      ],
    ]),
  ) as WalkCyclePngs;
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-world-test-compatibility",
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
      timestamp: "2026-07-28T00:00:00.000Z",
      idSuffix: "m05world",
    },
  );
  const views = {
    down: actorPng(),
    right: actorPng(1),
    up: actorPng(2),
    left: actorPng(3),
  };
  const concept = await createConceptCandidate(
    session.id,
    views.down,
    { source: "imported", originalFilename: "world-test-down.png" },
    {
      root,
      timestamp: "2026-07-28T00:01:00.000Z",
      idSuffix: "selected",
    },
  );
  const turnaround = await createTurnaroundCandidate(
    session.id,
    concept.id,
    views,
    { source: "imported", originalFilename: "world-test-turnaround" },
    {
      root,
      timestamp: "2026-07-28T00:02:00.000Z",
      idSuffix: "accepted",
    },
  );
  const frames = frameSet(views);
  const walkCycle = await createWalkCycleCandidate(
    session.id,
    turnaround.id,
    frames,
    { source: "imported", originalFilename: "world-test-walk" },
    {
      root,
      timestamp: "2026-07-28T00:03:00.000Z",
      idSuffix: "accepted",
    },
  );

  let missingSourceFailed = false;
  try {
    await createWorldTestCandidate(session.id, "missing-walk-cycle", {
      root,
    });
  } catch {
    missingSourceFailed = true;
  }
  assert(
    missingSourceFailed,
    "World Test unexpectedly accepted a missing Walk Cycle source.",
  );

  await Promise.all([
    server.connect(serverTransport),
    client.connect(clientTransport),
  ]);
  const createdResult = await client.callTool({
    name: "create_world_test_candidate",
    arguments: {
      sessionId: session.id,
      sourceWalkCycleId: walkCycle.id,
    },
  });
  assert(createdResult.isError !== true, "MCP rejected a valid World Test.");
  const created = parseWorldTestCandidate(
    JSON.parse(textPayload(createdResult)),
  );
  assert(
    created.sourceWalkCycle.walkCycleId === walkCycle.id &&
      created.sourceWalkCycle.acceptedBy === "user" &&
      created.sourceWalkCycle.frameSources.length === 16 &&
      created.previews.length === 16 &&
      created.preparation.additionalAiCost === false &&
      created.reviewStatus === "unreviewed" &&
      created.finalArtJudgment.status === "not_assessed",
    "World Test creation crossed the source, cost, or approval boundary.",
  );

  const listedResult = await client.callTool({
    name: "list_world_test_candidates",
    arguments: { sessionId: session.id },
  });
  const listed = JSON.parse(textPayload(listedResult)) as {
    candidates: WorldTestCandidate[];
  };
  assert(
    listed.candidates.length === 1 &&
      listed.candidates[0]?.id === created.id,
    "MCP list did not return the World Test.",
  );

  const fetchedResult = await client.callTool({
    name: "get_world_test_candidate",
    arguments: {
      sessionId: session.id,
      worldTestId: created.id,
    },
  });
  const fetched = JSON.parse(textPayload(fetchedResult)) as {
    candidate: WorldTestCandidate;
    previewPngBase64: Record<string, string>;
  };
  assert(
    Object.keys(fetched.previewPngBase64).length === 16,
    "MCP did not return all sixteen immutable previews.",
  );
  for (const preview of created.previews) {
    const key = `${preview.scene}/${preview.theme}`;
    const bytes = Buffer.from(fetched.previewPngBase64[key]!, "base64");
    assert(
      bytes.byteLength === preview.byteLength,
      `MCP changed ${key} preview bytes.`,
    );
  }

  const validationResult = await client.callTool({
    name: "validate_world_test_candidate",
    arguments: {
      sessionId: session.id,
      worldTestId: created.id,
    },
  });
  const validation = parseWorldTestValidationReport(
    JSON.parse(textPayload(validationResult)),
  );
  assert(
    validation.measurements.length === 256 &&
      validation.summary.pass + validation.summary.fail === 256 &&
      validation.summary.notAssessed === 0 &&
      validation.finalArtJudgment.status === "not_assessed" &&
      validation.finalArtJudgment.authority === "user",
    "World Test ground evidence or approval authority changed.",
  );

  const collisionOptions = {
    root,
    timestamp: "2026-07-28T00:30:00.000Z",
    idSuffix: "same0001",
    revision: 2,
  };
  const second = await createWorldTestCandidate(
    session.id,
    walkCycle.id,
    collisionOptions,
  );
  let collisionFailed = false;
  try {
    await createWorldTestCandidate(session.id, walkCycle.id, {
      ...collisionOptions,
      temporarySuffix: "collision-cleanup",
    });
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "World Test collision overwrote immutable previews.");
  const entries = await readdir(
    join(root, "sessions", session.id, "world-tests"),
  );
  assert(
    entries.length === 2 && !entries.some((name) => name.startsWith(".")),
    "Failed World Test creation left a partial directory.",
  );
  const secondPayload = await getWorldTestCandidatePayload(
    session.id,
    second.id,
    root,
  );
  assert(
    Object.keys(secondPayload.previewPngBytes).length === 16,
    "World Test collision changed the preserved candidate.",
  );

  const fixture = parseWorldTestCandidate(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let approvalRejected = false;
  try {
    parseWorldTestCandidate({
      ...fixture,
      finalArtJudgment: {
        ...fixture.finalArtJudgment,
        status: "approved",
      },
    });
  } catch {
    approvalRejected = true;
  }
  assert(
    approvalRejected,
    "World Test document accepted agent-writable final-art approval.",
  );

  console.log(
    "World Test compatibility passed (accepted Walk Cycle receipt, pinned references, sixteen immutable previews, 256 ground checks, failure cleanup).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
