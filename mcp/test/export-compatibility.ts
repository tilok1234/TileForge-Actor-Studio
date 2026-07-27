import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import {
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { createConceptCandidate } from "../src/candidates.js";
import {
  createExportCandidate,
  getExportCandidatePayload,
} from "../src/exports.js";
import { createActorStudioServer } from "../src/server.js";
import { createSession } from "../src/storage.js";
import { createTurnaroundCandidate } from "../src/turnarounds.js";
import {
  createWalkCycleCandidate,
  type WalkCyclePngs,
} from "../src/walk-cycles.js";
import { createWorldTestCandidate } from "../src/world-tests.js";
import {
  parseExportCandidate,
  parseExportMetadata,
  parseExportProvenance,
  type ExportCandidate,
} from "../../src/lib/studio/export.js";
import { parseExportValidationReport } from "../../src/lib/studio/export-validation.js";
import { type TurnaroundDirection } from "../../src/lib/studio/turnaround.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "export-candidate-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-m05-export-"));

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
      png.data[index] = 16 + offset;
      png.data[index + 1] = 28 + offset;
      png.data[index + 2] = 22 + offset;
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
        actorPng(4 + directionIndex),
        actorPng(8 + directionIndex),
        actorPng(12 + directionIndex),
      ],
    ]),
  ) as WalkCyclePngs;
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-export-compatibility",
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
      timestamp: "2026-07-28T01:00:00.000Z",
      idSuffix: "m05export",
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
    { source: "imported", originalFilename: "export-down.png" },
    {
      root,
      timestamp: "2026-07-28T01:01:00.000Z",
      idSuffix: "selected",
    },
  );
  const turnaround = await createTurnaroundCandidate(
    session.id,
    concept.id,
    views,
    { source: "imported", originalFilename: "export-turnaround" },
    {
      root,
      timestamp: "2026-07-28T01:02:00.000Z",
      idSuffix: "accepted",
    },
  );
  const walkCycle = await createWalkCycleCandidate(
    session.id,
    turnaround.id,
    frameSet(views),
    { source: "imported", originalFilename: "export-walk" },
    {
      root,
      timestamp: "2026-07-28T01:03:00.000Z",
      idSuffix: "accepted",
    },
  );
  const worldTest = await createWorldTestCandidate(
    session.id,
    walkCycle.id,
    {
      root,
      timestamp: "2026-07-28T01:04:00.000Z",
      idSuffix: "approved",
    },
  );

  let missingSourceFailed = false;
  try {
    await createExportCandidate(session.id, "missing-world-test", { root });
  } catch {
    missingSourceFailed = true;
  }
  assert(
    missingSourceFailed,
    "Export unexpectedly accepted a missing World Test source.",
  );

  await Promise.all([
    server.connect(serverTransport),
    client.connect(clientTransport),
  ]);
  const createdResult = await client.callTool({
    name: "create_export_candidate",
    arguments: {
      sessionId: session.id,
      sourceWorldTestId: worldTest.id,
    },
  });
  assert(
    createdResult.isError !== true,
    `MCP rejected a valid Export: ${textPayload(createdResult)}`,
  );
  const created = parseExportCandidate(
    JSON.parse(textPayload(createdResult)),
  );
  assert(
    created.approvedWorldTest.worldTestId === worldTest.id &&
      created.approvedWorldTest.approvedBy === "user" &&
      created.sourceWalkCycle.walkCycleId === walkCycle.id &&
      created.sourceWalkCycle.frameSources.length === 16 &&
      created.package.spriteSheet.width === 128 &&
      created.package.spriteSheet.height === 128 &&
      created.preparation.additionalAiCost === false &&
      created.status === "draft" &&
      created.publishing.status === "not_approved",
    "Export crossed its source, cost, draft, or publishing boundary.",
  );

  const listedResult = await client.callTool({
    name: "list_export_candidates",
    arguments: { sessionId: session.id },
  });
  const listed = JSON.parse(textPayload(listedResult)) as {
    candidates: ExportCandidate[];
  };
  assert(
    listed.candidates.length === 1 &&
      listed.candidates[0]?.id === created.id,
    "MCP list did not return the Export.",
  );

  const fetchedResult = await client.callTool({
    name: "get_export_candidate",
    arguments: { sessionId: session.id, exportId: created.id },
  });
  const fetched = JSON.parse(textPayload(fetchedResult)) as {
    candidate: ExportCandidate;
    spriteSheetPngBase64: string;
    metadata: unknown;
    provenance: unknown;
  };
  const sheetBytes = Buffer.from(fetched.spriteSheetPngBase64, "base64");
  const sheet = PNG.sync.read(sheetBytes, { checkCRC: true });
  const metadata = parseExportMetadata(fetched.metadata);
  const provenance = parseExportProvenance(fetched.provenance);
  assert(
    sheet.width === 128 &&
      sheet.height === 128 &&
      metadata.animation.directions.join("/") === "down/right/up/left" &&
      metadata.frames.length === 16 &&
      provenance.exportId === created.id &&
      provenance.approvedWorldTest.approvedBy === "user" &&
      provenance.publishing.status === "not_approved",
    "MCP changed the Export sheet, metadata, or provenance.",
  );

  const validationResult = await client.callTool({
    name: "validate_export_candidate",
    arguments: { sessionId: session.id, exportId: created.id },
  });
  const validation = parseExportValidationReport(
    JSON.parse(textPayload(validationResult)),
  );
  assert(
    validation.summary.pass === 7 &&
      validation.summary.fail === 0 &&
      validation.summary.notAssessed === 0 &&
      validation.publishing.status === "not_approved" &&
      validation.publishing.authority === "user",
    "Export validation or publishing authority changed.",
  );

  const collisionOptions = {
    root,
    timestamp: "2026-07-28T01:30:00.000Z",
    idSuffix: "same0001",
    revision: 2,
  };
  const second = await createExportCandidate(
    session.id,
    worldTest.id,
    collisionOptions,
  );
  let collisionFailed = false;
  try {
    await createExportCandidate(session.id, worldTest.id, {
      ...collisionOptions,
      temporarySuffix: "collision-cleanup",
    });
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "Export collision overwrote an immutable package.");
  const entries = await readdir(
    join(root, "sessions", session.id, "exports"),
  );
  assert(
    entries.length === 2 && !entries.some((name) => name.startsWith(".")),
    "Failed Export creation left a partial directory.",
  );
  await getExportCandidatePayload(session.id, second.id, root);

  await writeFile(
    join(
      root,
      "sessions",
      session.id,
      "exports",
      second.id,
      "metadata.json",
    ),
    "{}\n",
    "utf8",
  );
  let tamperFailed = false;
  try {
    await getExportCandidatePayload(session.id, second.id, root);
  } catch {
    tamperFailed = true;
  }
  assert(tamperFailed, "Export accepted tampered package metadata.");

  const fixture = parseExportCandidate(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let publishingRejected = false;
  try {
    parseExportCandidate({
      ...fixture,
      publishing: {
        ...fixture.publishing,
        status: "approved",
      },
    });
  } catch {
    publishingRejected = true;
  }
  assert(
    publishingRejected,
    "Export document accepted agent-writable publishing approval.",
  );

  console.log(
    "Export compatibility passed (user-approved World Test receipt, immutable 4 x 4 sheet package, local provenance, publishing boundary, collision and tamper cleanup).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
