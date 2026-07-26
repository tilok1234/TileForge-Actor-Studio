import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import {
  cp,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseActorBrief } from "../../src/lib/studio/brief.js";
import { parseStudioSession } from "../../src/lib/studio/session.js";
import { createActorStudioServer } from "../src/server.js";
import { createSession, getSession } from "../src/storage.js";
import { textPayload } from "./result.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const fixturePath = join(repositoryRoot, "tests", "fixtures", "session-v1.json");
const root = await mkdtemp(join(tmpdir(), "tfas-m01-"));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer({ workspaceRoot: root });
const client = new Client({
  name: "tileforge-actor-studio-session-compatibility",
  version: "0.1.0",
});

try {
  await Promise.all([server.connect(serverTransport), client.connect(clientTransport)]);

  const invalid = await client.callTool({
    name: "create_sprite_session",
    arguments: {
      name: " ",
      kind: "mob",
      description: "A valid description.",
    },
  });
  assert(invalid.isError === true, "MCP accepted an incomplete brief.");
  const rootEntries = await readdir(root);
  assert(rootEntries.length === 0, "Invalid brief created partial storage.");

  const normalized = parseActorBrief({
    name: "  Longname Keeper  ",
    kind: "npc",
    description: "  Watches the old road.  ",
  });
  assert(normalized.name === "Longname Keeper", "Shared brief validation did not trim.");

  const brief = {
    name: "A".repeat(80),
    kind: "npc" as const,
    description: "A valid maximum-name compatibility actor.",
  };
  const identity = {
    root,
    timestamp: "2026-07-26T21:02:00.000Z",
    idSuffix: "same0001",
  };
  const created = await createSession(brief, identity);
  const reread = await getSession(created.id, root);
  assert(reread.id === created.id, "MCP could not read its durable session.");
  assert(created.id.length <= 96, "Maximum-length brief produced an unreadable id.");

  let collisionFailed = false;
  try {
    await createSession(brief, {
      ...identity,
      temporarySuffix: "collision-cleanup",
    });
  } catch {
    collisionFailed = true;
  }
  assert(collisionFailed, "Session identity collision unexpectedly overwrote a session.");
  const sessionEntries = await readdir(join(root, "sessions"));
  assert(
    sessionEntries.length === 1 && !sessionEntries.some((name) => name.startsWith(".")),
    "Failed atomic creation left a partial session directory.",
  );

  const fixture = parseStudioSession(
    JSON.parse(await readFile(fixturePath, "utf8")),
  );
  let approvalFieldRejected = false;
  try {
    parseStudioSession({ ...fixture, approved: true });
  } catch {
    approvalFieldRejected = true;
  }
  assert(
    approvalFieldRejected,
    "Session compatibility accepted an agent-writable approval field.",
  );
  const fixtureDirectory = join(root, "sessions", fixture.id);
  await mkdir(fixtureDirectory, { recursive: true });
  await cp(fixturePath, join(fixtureDirectory, "session.json"));
  await mkdir(join(fixtureDirectory, "candidates"));

  const desktopSession = await client.callTool({
    name: "get_sprite_session",
    arguments: { sessionId: fixture.id },
  });
  const parsedDesktopSession = parseStudioSession(
    JSON.parse(textPayload(desktopSession)),
  );
  assert(
    parsedDesktopSession.id === fixture.id &&
      parsedDesktopSession.contractId === fixture.contractId,
    "MCP could not read the desktop-compatible session fixture.",
  );

  console.log(
    "Session compatibility passed (shared validation, atomic failure cleanup, desktop/MCP document).",
  );
} finally {
  await client.close();
  await server.close();
  await rm(root, { recursive: true, force: true });
}
