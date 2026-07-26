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
  getWalkCycleCandidatePayload,
  type WalkCyclePngs,
} from "../src/walk-cycles.js";
import {
  parseWalkCycleCandidate,
  type WalkCycleCandidate,
} from "../../src/lib/studio/walk-cycle.js";
import { parseWalkCycleValidationReport } from "../../src/lib/studio/walk-cycle-validation.js";
import {
  type TurnaroundDirection,
} from "../../src/lib/studio/turnaround.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "walk-cycle-candidate-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-m04-walk-cycle-"));

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

function frameSet(
  views: Record<TurnaroundDirection, Uint8Array>,
): WalkCyclePngs {
  return Object.fromEntries(
    Object.entries(views).map(([direction, frameZero], directionIndex) => [
      direction,
      [
        frameZero,
        viewPng(8 + directionIndex * 3),
        viewPng(9 + directionIndex * 3),
        viewPng(10 + directionIndex * 3),
      ],
    ]),
  ) as WalkCyclePngs;
}

function base64Frames(
  frames: WalkCyclePngs,
): Record<TurnaroundDirection, string[]> {
  return Object.fromEntries(
    Object.entries(frames).map(([direction, directionFrames]) => [
      direction,
      directionFrames.map((bytes) => Buffer.from(bytes).toString("base64")),
    ]),
  ) as Record<TurnaroundDirection, string[]>;
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-walk-cycle-compatibility",
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
      timestamp: "2026-07-27T23:00:00.000Z",
      idSuffix: "m04walk1",
    },
  );
  const views = {
    down: viewPng(),
    right: viewPng(1),
    up: viewPng(2),
    left: viewPng(3),
  };
  const concept = await createConceptCandidate(
    session.id,
    views.down,
    {
      source: "generated",
      provider: "subscription-image-tool",
    },
    {
      root,
      timestamp: "2026-07-27T23:01:00.000Z",
      idSuffix: "selected",
    },
  );
  const turnaround = await createTurnaroundCandidate(
    session.id,
    concept.id,
    views,
    {
      source: "generated",
      provider: "subscription-image-tool",
    },
    {
      root,
      timestamp: "2026-07-27T23:02:00.000Z",
      idSuffix: "accepted",
    },
  );
  await Promise.all([
    server.connect(serverTransport),
    client.connect(clientTransport),
  ]);

  const frames = frameSet(views);
  for (const [label, invalidFrames] of [
    [
      "frame-zero replacement",
      { ...frames, down: [viewPng(20), ...frames.down.slice(1)] },
    ],
    [
      "missing frame",
      { ...frames, right: frames.right.slice(0, 3) },
    ],
    [
      "invalid frame dimensions",
      {
        ...frames,
        up: [
          frames.up[0]!,
          viewPng(20, 31, 32),
          frames.up[2]!,
          frames.up[3]!,
        ],
      },
    ],
  ] as const) {
    let failed = false;
    try {
      await createWalkCycleCandidate(
        session.id,
        turnaround.id,
        invalidFrames as WalkCyclePngs,
        {
          source: "imported",
          originalFilename: label,
        },
        { root },
      );
    } catch {
      failed = true;
    }
    assert(failed, `${label} unexpectedly created a Walk Cycle.`);
  }
  const walkCyclesRoot = join(
    root,
    "sessions",
    session.id,
    "walk-cycles",
  );
  const rejectedEntries = await readdir(walkCyclesRoot).catch(
    (error: NodeJS.ErrnoException) => {
      if (error.code === "ENOENT") {
        return [];
      }
      throw error;
    },
  );
  assert(
    rejectedEntries.length === 0,
    "Rejected Walk Cycle intake left partial storage.",
  );

  const createdResult = await client.callTool({
    name: "create_walk_cycle_candidate",
    arguments: {
      sessionId: session.id,
      sourceTurnaroundId: turnaround.id,
      pngBase64: base64Frames(frames),
      provenance: {
        source: "imported",
        originalFilename: "mirelight-walk-r1",
      },
    },
  });
  assert(createdResult.isError !== true, "MCP rejected a valid Walk Cycle.");
  const created = parseWalkCycleCandidate(
    JSON.parse(textPayload(createdResult)),
  );
  assert(
    created.sourceTurnaround.turnaroundId === turnaround.id &&
      created.sourceTurnaround.acceptedBy === "user" &&
      created.reviewStatus === "unreviewed" &&
      created.motionJudgment.status === "not_assessed" &&
      created.frameDurationMs === 300 &&
      created.frames.length === 16,
    "Walk Cycle creation crossed a user-owned gate or contract boundary.",
  );

  const listedResult = await client.callTool({
    name: "list_walk_cycle_candidates",
    arguments: { sessionId: session.id },
  });
  const listed = JSON.parse(textPayload(listedResult)) as {
    candidates: WalkCycleCandidate[];
  };
  assert(
    listed.candidates.length === 1 &&
      listed.candidates[0]?.id === created.id,
    "MCP list did not return the Walk Cycle.",
  );

  const fetchedResult = await client.callTool({
    name: "get_walk_cycle_candidate",
    arguments: {
      sessionId: session.id,
      walkCycleId: created.id,
    },
  });
  const fetched = JSON.parse(textPayload(fetchedResult)) as {
    candidate: WalkCycleCandidate;
    pngBase64: Record<TurnaroundDirection, string[]>;
  };
  for (const direction of ["down", "right", "up", "left"] as const) {
    for (let frameIndex = 0; frameIndex < 4; frameIndex += 1) {
      assert(
        Buffer.from(
          fetched.pngBase64[direction]![frameIndex]!,
          "base64",
        ).equals(Buffer.from(frames[direction][frameIndex]!)),
        `MCP changed immutable ${direction} frame ${frameIndex} bytes.`,
      );
    }
  }

  const validationResult = await client.callTool({
    name: "validate_walk_cycle_candidate",
    arguments: {
      sessionId: session.id,
      walkCycleId: created.id,
    },
  });
  const validation = parseWalkCycleValidationReport(
    JSON.parse(textPayload(validationResult)),
  );
  assert(
    validation.summary.pass === 96 &&
      validation.summary.fail === 0 &&
      validation.summary.notAssessed === 16 &&
      validation.motionJudgment.status === "not_assessed" &&
      validation.motionJudgment.authority === "user",
    "Walk Cycle validation totals or authority changed.",
  );

  const collisionOptions = {
    root,
    timestamp: "2026-07-27T23:31:00.000Z",
    idSuffix: "same0001",
    revision: 2,
  };
  const second = await createWalkCycleCandidate(
    session.id,
    turnaround.id,
    frames,
    {
      source: "imported",
      originalFilename: "collision-one",
    },
    collisionOptions,
  );
  let collisionFailed = false;
  try {
    await createWalkCycleCandidate(
      session.id,
      turnaround.id,
      frames,
      {
        source: "imported",
        originalFilename: "collision-two",
      },
      { ...collisionOptions, temporarySuffix: "collision-cleanup" },
    );
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "Walk Cycle collision overwrote immutable bytes.");
  const entries = await readdir(walkCyclesRoot);
  assert(
    entries.length === 2 && !entries.some((name) => name.startsWith(".")),
    "Failed Walk Cycle creation left a partial directory.",
  );
  const secondPayload = await getWalkCycleCandidatePayload(
    session.id,
    second.id,
    root,
  );
  assert(
    Buffer.from(secondPayload.pngBytes.down[0]!).equals(
      Buffer.from(views.down),
    ),
    "Collision changed the accepted Turnaround frame-zero bytes.",
  );

  const fixture = parseWalkCycleCandidate(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let approvalRejected = false;
  try {
    parseWalkCycleCandidate({
      ...fixture,
      motionJudgment: {
        ...fixture.motionJudgment,
        status: "accepted",
      },
    });
  } catch {
    approvalRejected = true;
  }
  assert(
    approvalRejected,
    "Walk Cycle document accepted an agent-writable motion approval.",
  );

  console.log(
    "Walk Cycle compatibility passed (accepted Turnaround receipt, sixteen immutable frames, MCP parity, structural evidence, failure cleanup).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
