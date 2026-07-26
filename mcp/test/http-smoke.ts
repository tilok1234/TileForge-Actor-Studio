import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import { textPayload } from "./result.js";

const endpoint = new URL(process.env.TFAS_MCP_URL ?? "http://127.0.0.1:7331/mcp");
const client = new Client({
  name: "tileforge-actor-studio-http-smoke",
  version: "0.1.0",
});
const transport = new StreamableHTTPClientTransport(endpoint);

try {
  await client.connect(transport);
  const tools = await client.listTools();
  if (!tools.tools.some((tool) => tool.name === "get_studio_contract")) {
    throw new Error("HTTP transport did not expose the studio contract tool.");
  }

  const result = await client.callTool({
    name: "get_studio_contract",
    arguments: {},
  });
  if (!textPayload(result).includes("tileforge-actor-32-v1")) {
    throw new Error("HTTP contract response was incomplete.");
  }

  console.log(`MCP HTTP smoke passed (${tools.tools.length} tools at ${endpoint.href}).`);
} finally {
  await client.close();
}
