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
import type { ActorBrief, StudioSession } from "../../src/lib/studio/types.js";
import {
  createStudioSession,
  parseStudioSession,
  SESSION_ID_MAX_LENGTH,
} from "../../src/lib/studio/session.js";
import { workspaceRoot } from "./workspace.js";

export { workspaceRoot };

function sessionsRoot(root: string): string {
  return join(root, "sessions");
}

async function ensureStorage(root: string): Promise<void> {
  await mkdir(sessionsRoot(root), { recursive: true });
}

function assertSessionId(sessionId: string): string {
  if (
    sessionId.length > SESSION_ID_MAX_LENGTH ||
    !/^[a-z0-9][a-z0-9-]{2,95}$/i.test(sessionId)
  ) {
    throw new Error("Invalid session id.");
  }
  return sessionId;
}

function sessionFile(root: string, sessionId: string): string {
  return join(sessionsRoot(root), assertSessionId(sessionId), "session.json");
}

interface CreateSessionOptions {
  root?: string;
  timestamp?: string;
  idSuffix?: string;
  temporarySuffix?: string;
}

export async function createSession(
  brief: ActorBrief,
  options: CreateSessionOptions = {},
): Promise<StudioSession> {
  const root = options.root ?? workspaceRoot;
  const session = createStudioSession(brief, {
    timestamp: options.timestamp,
    idSuffix: options.idSuffix ?? randomUUID().slice(0, 8),
  });
  await ensureStorage(root);

  const finalDirectory = join(sessionsRoot(root), session.id);
  const temporaryDirectory = join(
    sessionsRoot(root),
    `.${session.id}.${options.temporarySuffix ?? randomUUID()}.tmp`,
  );
  await mkdir(temporaryDirectory, { recursive: false });

  try {
    await writeFile(
      join(temporaryDirectory, "session.json"),
      `${JSON.stringify(session, null, 2)}\n`,
      { encoding: "utf8", flag: "wx" },
    );
    await mkdir(join(temporaryDirectory, "candidates"), { recursive: false });
    await rename(temporaryDirectory, finalDirectory);
    return session;
  } catch (error) {
    await rm(temporaryDirectory, { recursive: true, force: true });
    throw error;
  }
}

export async function getSession(
  sessionId: string,
  root = workspaceRoot,
): Promise<StudioSession> {
  const raw = await readFile(sessionFile(root, sessionId), "utf8");
  return parseStudioSession(JSON.parse(raw));
}

export async function listSessions(root = workspaceRoot): Promise<StudioSession[]> {
  await ensureStorage(root);
  const entries = await readdir(sessionsRoot(root), { withFileTypes: true });
  const sessions = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && !entry.name.startsWith("."))
      .map(async (entry) => {
        try {
          return await getSession(entry.name, root);
        } catch {
          return null;
        }
      }),
  );

  return sessions
    .filter((session): session is StudioSession => session !== null)
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
}
