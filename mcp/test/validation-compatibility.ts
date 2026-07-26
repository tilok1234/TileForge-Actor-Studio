import { createHash } from "node:crypto";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import type { ConceptCandidate } from "../../src/lib/studio/candidate.js";
import { validateConceptCandidatePng } from "../../src/lib/studio/validate-candidate.js";
import {
  parseValidationReport,
  type ValidationReport,
} from "../../src/lib/studio/validation.js";
import { createConceptCandidate } from "../src/candidates.js";
import { createActorStudioServer } from "../src/server.js";
import { createSession } from "../src/storage.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(
  repositoryRoot,
  "tests",
  "fixtures",
  "validation-report-v1.json",
);
const root = await mkdtemp(join(tmpdir(), "tfas-m03-"));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function actorPng(
  mode: "passing" | "failing",
  width = 32,
  height = 32,
): Uint8Array {
  const png = new PNG({ width, height });
  png.data.fill(0);
  const firstY = mode === "passing" ? 5 : 12;
  let colorIndex = 0;

  for (let y = firstY; y <= Math.min(28, height - 1); y += 1) {
    for (let x = 10; x <= Math.min(21, width - 1); x += 1) {
      const index = (y * width + x) * 4;
      if (mode === "passing") {
        const light = y % 2 === 0;
        png.data[index] = light ? 104 : 42;
        png.data[index + 1] = light ? 198 : 74;
        png.data[index + 2] = light ? 178 : 55;
      } else {
        const color = colorIndex % 20;
        png.data[index] = color * 11;
        png.data[index + 1] = 40 + color * 7;
        png.data[index + 2] = 220 - color * 9;
        colorIndex += 1;
      }
      png.data[index + 3] = 255;
    }
  }

  if (mode === "failing") {
    png.data[(12 * width + 10) * 4 + 3] = 128;
    png.data[(28 * width + 16) * 4 + 3] = 0;
    const edge = (20 * width) * 4;
    png.data[edge] = 42;
    png.data[edge + 1] = 74;
    png.data[edge + 2] = 55;
    png.data[edge + 3] = 255;
    const bottomEdge = (31 * width + 15) * 4;
    png.data[bottomEdge] = 42;
    png.data[bottomEdge + 1] = 74;
    png.data[bottomEdge + 2] = 55;
    png.data[bottomEdge + 3] = 255;
  }

  return PNG.sync.write(png);
}

async function validateThroughMcp(
  client: Client,
  sessionId: string,
  candidateId: string,
): Promise<ValidationReport> {
  const result = await client.callTool({
    name: "validate_concept_candidate",
    arguments: { sessionId, candidateId },
  });
  assert(result.isError !== true, "MCP rejected a readable candidate.");
  return parseValidationReport(JSON.parse(textPayload(result)));
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-validation-compatibility",
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
      timestamp: "2026-07-26T22:00:00.000Z",
      idSuffix: "m03test1",
    },
  );
  await Promise.all([server.connect(serverTransport), client.connect(clientTransport)]);

  const passingBytes = actorPng("passing");
  const passingCandidate = await createConceptCandidate(
    session.id,
    passingBytes,
    {
      source: "imported",
      originalFilename: "passing-actor.png",
    },
    {
      root,
      timestamp: "2026-07-26T22:01:00.000Z",
      idSuffix: "passing1",
    },
  );
  const passingReport = await validateThroughMcp(
    client,
    session.id,
    passingCandidate.id,
  );
  assert(
    passingReport.summary.pass === 6 &&
      passingReport.summary.fail === 0 &&
      passingReport.summary.notAssessed === 1,
    "Passing candidate produced an unexpected summary.",
  );
  assert(
    passingReport.visualJudgment.status === "not_assessed" &&
      passingReport.visualJudgment.authority === "user",
    "Structural validation crossed the human approval boundary.",
  );

  const fixture = parseValidationReport(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  assert(
    JSON.stringify({
      results: passingReport.results,
      summary: passingReport.summary,
      visualJudgment: passingReport.visualJudgment,
    }) ===
      JSON.stringify({
        results: fixture.results,
        summary: fixture.summary,
        visualJudgment: fixture.visualJudgment,
      }),
    "TypeScript validation semantics drifted from the shared fixture.",
  );

  const failingCandidate = await createConceptCandidate(
    session.id,
    actorPng("failing"),
    {
      source: "imported",
      originalFilename: "failing-actor.png",
    },
    {
      root,
      timestamp: "2026-07-26T22:02:00.000Z",
      idSuffix: "failing1",
    },
  );
  const failingReport = await validateThroughMcp(
    client,
    session.id,
    failingCandidate.id,
  );
  const failedRules = failingReport.results
    .filter((result) => result.status === "fail")
    .map((result) => result.id);
  assert(
    JSON.stringify(failedRules) ===
      JSON.stringify([
        "hard_alpha",
        "actor_height",
        "foot_anchor",
        "palette_max_colors",
        "frame_edge_clipping",
      ]),
    `Independent failure precedence changed: ${failedRules.join(", ")}`,
  );
  assert(
    failingReport.summary.pass === 1 &&
      failingReport.summary.fail === 5 &&
      failingReport.summary.notAssessed === 1,
    "Failing candidate summary is incorrect.",
  );
  assert(
    failingReport.results.find((result) => result.id === "frame_edge_clipping")
      ?.observed === "2 pixels on bottom, left",
    "Edge evidence order drifted between validators.",
  );

  const narrowBytes = actorPng("passing", 31, 32);
  const narrowCandidate: ConceptCandidate = {
    ...passingCandidate,
    sha256: createHash("sha256").update(narrowBytes).digest("hex"),
    byteLength: narrowBytes.byteLength,
  };
  const narrowReport = validateConceptCandidatePng(
    narrowCandidate,
    narrowBytes,
  );
  assert(
    narrowReport.results[0]?.status === "fail",
    "Canvas validation trusted candidate metadata instead of decoded bytes.",
  );

  const candidateDirectory = join(
    root,
    "sessions",
    session.id,
    "candidates",
    passingCandidate.id,
  );
  assert(
    JSON.stringify((await readdir(candidateDirectory)).sort()) ===
      JSON.stringify(["candidate.json", "source.png"]),
    "Read-only validation wrote into the immutable candidate directory.",
  );
  assert(
    Buffer.from(await readFile(join(candidateDirectory, "source.png"))).equals(
      Buffer.from(passingBytes),
    ),
    "Validation changed immutable source bytes.",
  );

  let approvalFieldRejected = false;
  try {
    parseValidationReport({ ...fixture, approved: true });
  } catch {
    approvalFieldRejected = true;
  }
  assert(
    approvalFieldRejected,
    "Validation report accepted an agent-writable approval field.",
  );

  console.log(
    "Validation compatibility passed (7 rules, independent failures, immutable read, human boundary).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
