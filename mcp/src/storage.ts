import { randomUUID } from "node:crypto";
import {
  mkdir,
  readFile,
  readdir,
  writeFile,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { ActorBrief, StudioSession } from "../../src/lib/studio/types.js";
import { createStudioSession } from "../../src/lib/studio/session.js";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");

export const workspaceRoot = resolve(
  process.env.TFAS_WORKSPACE ?? join(repositoryRoot, ".studio"),
);

const sessionsRoot = join(workspaceRoot, "sessions");

async function ensureStorage(): Promise<void> {
  await mkdir(sessionsRoot, { recursive: true });
}

function assertSessionId(sessionId: string): string {
  if (!/^[a-z0-9][a-z0-9-]{2,96}$/i.test(sessionId)) {
    throw new Error("Invalid session id.");
  }
  return sessionId;
}

function sessionFile(sessionId: string): string {
  return join(sessionsRoot, assertSessionId(sessionId), "session.json");
}

export async function createSession(brief: ActorBrief): Promise<StudioSession> {
  await ensureStorage();

  const base = createStudioSession(brief);
  const session: StudioSession = {
    ...base,
    id: `${base.id}-${randomUUID().slice(0, 8)}`,
  };
  const directory = join(sessionsRoot, session.id);
  await mkdir(directory, { recursive: false });
  await writeFile(
    join(directory, "session.json"),
    `${JSON.stringify(session, null, 2)}\n`,
    { encoding: "utf8", flag: "wx" },
  );
  await mkdir(join(directory, "candidates"), { recursive: false });
  return session;
}

export async function getSession(sessionId: string): Promise<StudioSession> {
  const raw = await readFile(sessionFile(sessionId), "utf8");
  return JSON.parse(raw) as StudioSession;
}

export async function listSessions(): Promise<StudioSession[]> {
  await ensureStorage();
  const entries = await readdir(sessionsRoot, { withFileTypes: true });
  const sessions = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory())
      .map(async (entry) => {
        try {
          return await getSession(entry.name);
        } catch {
          return null;
        }
      }),
  );

  return sessions
    .filter((session): session is StudioSession => session !== null)
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
}
