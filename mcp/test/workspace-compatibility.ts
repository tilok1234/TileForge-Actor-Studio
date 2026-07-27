import { join, resolve } from "node:path";
import { resolveWorkspaceRoot } from "../src/workspace.js";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

const fallbackRoot = resolve("C:\\source\\TileForge-Actor-Studio");
const localAppData = resolve("C:\\Users\\artist\\AppData\\Local");
const redirected = resolve("D:\\TileForge Workspace");

assert(
  resolveWorkspaceRoot(
    { TFAS_WORKSPACE: redirected, LOCALAPPDATA: localAppData },
    "win32",
    fallbackRoot,
  ) === redirected,
  "TFAS_WORKSPACE did not retain highest precedence.",
);

assert(
  resolveWorkspaceRoot(
    { LOCALAPPDATA: localAppData },
    "win32",
    fallbackRoot,
  ) ===
    join(localAppData, "TileForge", "Actor Studio", ".studio"),
  "Windows default workspace did not resolve under per-user local app data.",
);

assert(
  resolveWorkspaceRoot({}, "linux", fallbackRoot) ===
    join(fallbackRoot, ".studio"),
  "Non-Windows fallback no longer resolves to the repository workspace.",
);

console.log(
  "Workspace compatibility passed (override, packaged Windows default, repository fallback).",
);
