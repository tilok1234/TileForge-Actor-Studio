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
import { createConceptCandidate } from "../src/candidates.js";
import { createActorStudioServer } from "../src/server.js";
import { createSession } from "../src/storage.js";
import {
  createTurnaroundCandidate,
  getTurnaroundCandidatePayload,
} from "../src/turnarounds.js";
import {
  parseTurnaroundCandidate,
  type TurnaroundCandidate,
  type TurnaroundDirection,
} from "../../src/lib/studio/turnaround.js";
import { parseTurnaroundValidationReport } from "../../src/lib/studio/turnaround-validation.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "turnaround-candidate-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-m04-turnaround-"));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function viewPng(
  colorOffset = 0,
  width = 32,
  height = 32,
): Uint8Array {
  const png = new PNG({ width, height });
  for (let index = 0; index < png.data.length; index += 4) {
    png.data[index + 3] = 0;
  }
  for (let y = 5; y <= Math.min(28, height - 1); y += 1) {
    for (let x = 10; x <= Math.min(21, width - 1); x += 1) {
      const index = (y * width + x) * 4;
      png.data[index] = 42 + colorOffset;
      png.data[index + 1] = 74 + colorOffset;
      png.data[index + 2] = 55 + colorOffset;
      png.data[index + 3] = 255;
    }
  }
  return PNG.sync.write(png);
}

function base64Views(
  views: Record<TurnaroundDirection, Uint8Array>,
): Record<TurnaroundDirection, string> {
  return Object.fromEntries(
    Object.entries(views).map(([direction, bytes]) => [
      direction,
      Buffer.from(bytes).toString("base64"),
    ]),
  ) as Record<TurnaroundDirection, string>;
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-turnaround-compatibility",
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
      timestamp: "2026-07-27T20:00:00.000Z",
      idSuffix: "m04turn1",
    },
  );
  const down = viewPng();
  const concept = await createConceptCandidate(
    session.id,
    down,
    {
      source: "generated",
      provider: "subscription-image-tool",
    },
    {
      root,
      timestamp: "2026-07-27T20:01:00.000Z",
      idSuffix: "selected",
    },
  );
  await Promise.all([
    server.connect(serverTransport),
    client.connect(clientTransport),
  ]);

  const views = {
    down,
    right: viewPng(1),
    up: viewPng(2),
    left: viewPng(3),
  };

  for (const [label, invalidViews] of [
    ["down replacement", { ...views, down: viewPng(4) }],
    ["invalid right dimensions", { ...views, right: viewPng(1, 31, 32) }],
  ] as const) {
    let failed = false;
    try {
      await createTurnaroundCandidate(
        session.id,
        concept.id,
        invalidViews,
        {
          source: "generated",
          provider: "subscription-image-tool",
        },
        { root },
      );
    } catch {
      failed = true;
    }
    assert(failed, `${label} unexpectedly created a Turnaround.`);
  }
  const turnaroundsRoot = join(
    root,
    "sessions",
    session.id,
    "turnarounds",
  );
  const rejectedEntries = await readdir(turnaroundsRoot).catch(
    (error: NodeJS.ErrnoException) => {
      if (error.code === "ENOENT") {
        return [];
      }
      throw error;
    },
  );
  assert(
    rejectedEntries.length === 0,
    "Rejected Turnaround intake left partial storage.",
  );

  const createdResult = await client.callTool({
    name: "create_turnaround_candidate",
    arguments: {
      sessionId: session.id,
      sourceConceptId: concept.id,
      pngBase64: base64Views(views),
      provenance: {
        source: "generated",
        provider: "subscription-image-tool",
        model: "built-in",
      },
    },
  });
  assert(createdResult.isError !== true, "MCP rejected a valid Turnaround.");
  const created = parseTurnaroundCandidate(
    JSON.parse(textPayload(createdResult)),
  );
  assert(
    created.sourceSelection.candidateId === concept.id &&
      created.sourceSelection.selectedBy === "user" &&
      created.reviewStatus === "unreviewed" &&
      created.identityJudgment.status === "not_assessed",
    "Turnaround creation crossed a user-owned approval boundary.",
  );

  const listedResult = await client.callTool({
    name: "list_turnaround_candidates",
    arguments: { sessionId: session.id },
  });
  const listed = JSON.parse(textPayload(listedResult)) as {
    candidates: TurnaroundCandidate[];
  };
  assert(
    listed.candidates.length === 1 &&
      listed.candidates[0]?.id === created.id,
    "MCP list did not return the Turnaround.",
  );

  const fetchedResult = await client.callTool({
    name: "get_turnaround_candidate",
    arguments: {
      sessionId: session.id,
      turnaroundId: created.id,
    },
  });
  const fetched = JSON.parse(textPayload(fetchedResult)) as {
    candidate: TurnaroundCandidate;
    pngBase64: Record<TurnaroundDirection, string>;
  };
  for (const direction of ["down", "right", "up", "left"] as const) {
    assert(
      Buffer.from(fetched.pngBase64[direction], "base64").equals(
        Buffer.from(views[direction]),
      ),
      `MCP changed the immutable ${direction} PNG bytes.`,
    );
  }

  const validationResult = await client.callTool({
    name: "validate_turnaround_candidate",
    arguments: {
      sessionId: session.id,
      turnaroundId: created.id,
    },
  });
  const validation = parseTurnaroundValidationReport(
    JSON.parse(textPayload(validationResult)),
  );
  assert(
    validation.summary.pass === 24 &&
      validation.summary.fail === 0 &&
      validation.summary.notAssessed === 4 &&
      validation.identityJudgment.status === "not_assessed" &&
      validation.identityJudgment.authority === "user",
    "Turnaround validation totals or authority changed.",
  );

  const collisionOptions = {
    root,
    timestamp: "2026-07-27T20:31:00.000Z",
    idSuffix: "same0001",
    revision: 2,
  };
  const second = await createTurnaroundCandidate(
    session.id,
    concept.id,
    views,
    {
      source: "generated",
      provider: "subscription-image-tool",
    },
    collisionOptions,
  );
  let collisionFailed = false;
  try {
    await createTurnaroundCandidate(
      session.id,
      concept.id,
      views,
      {
        source: "generated",
        provider: "subscription-image-tool",
      },
      { ...collisionOptions, temporarySuffix: "collision-cleanup" },
    );
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "Turnaround collision overwrote immutable bytes.");
  const entries = await readdir(turnaroundsRoot);
  assert(
    entries.length === 2 && !entries.some((name) => name.startsWith(".")),
    "Failed Turnaround creation left a partial directory.",
  );
  const secondPayload = await getTurnaroundCandidatePayload(
    session.id,
    second.id,
    root,
  );
  assert(
    Buffer.from(secondPayload.pngBytes.down).equals(Buffer.from(down)),
    "Collision changed the selected down-view bytes.",
  );

  const fixture = parseTurnaroundCandidate(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let approvalRejected = false;
  try {
    parseTurnaroundCandidate({
      ...fixture,
      identityJudgment: {
        ...fixture.identityJudgment,
        status: "accepted",
      },
    });
  } catch {
    approvalRejected = true;
  }
  assert(
    approvalRejected,
    "Turnaround document accepted an agent-writable identity approval.",
  );

  console.log(
    "Turnaround compatibility passed (user-selected source, four immutable views, MCP parity, structural evidence, failure cleanup).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
