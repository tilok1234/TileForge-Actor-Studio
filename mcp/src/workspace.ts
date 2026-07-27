import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const moduleDir = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(moduleDir, "../..");
const WINDOWS_VENDOR_DIRECTORY = "TileForge";
const WINDOWS_PRODUCT_DIRECTORY = "Actor Studio";

export function resolveWorkspaceRoot(
  environment: NodeJS.ProcessEnv = process.env,
  platform = process.platform,
  fallbackRoot = repositoryRoot,
): string {
  const override = environment.TFAS_WORKSPACE?.trim();
  if (override) {
    return resolve(override);
  }

  const localAppData = environment.LOCALAPPDATA?.trim();
  if (platform === "win32" && localAppData) {
    return resolve(
      join(
        localAppData,
        WINDOWS_VENDOR_DIRECTORY,
        WINDOWS_PRODUCT_DIRECTORY,
        ".studio",
      ),
    );
  }

  return resolve(join(fallbackRoot, ".studio"));
}

export const workspaceRoot = resolveWorkspaceRoot();
