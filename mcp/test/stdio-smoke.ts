import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { textPayload } from "./result.js";

const client = new Client({
  name: "tileforge-actor-studio-stdio-smoke",
  version: "0.1.0",
});
const transport = new StdioClientTransport({
  command: process.execPath,
  args: ["--import", "tsx", "mcp/src/index.ts", "--transport", "stdio"],
  cwd: process.cwd(),
  stderr: "pipe",
});

try {
  await client.connect(transport);
  const tools = await client.listTools();
  if (!tools.tools.some((tool) => tool.name === "compile_actor_prompt")) {
    throw new Error("Stdio transport did not expose the prompt compiler.");
  }

  const result = await client.callTool({
    name: "compile_actor_prompt",
    arguments: {
      name: "Mirelight Pilgrim",
      kind: "npc",
      description: "A quiet reed-cloaked marsh pilgrim carrying a blue-green lantern.",
    },
  });
  if (!textPayload(result).includes("32x32")) {
    throw new Error("Stdio prompt response omitted the frame boundary.");
  }

  console.log(`MCP stdio smoke passed (${tools.tools.length} tools).`);
} finally {
  await client.close();
}
