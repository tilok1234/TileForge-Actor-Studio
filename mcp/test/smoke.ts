import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createActorStudioServer } from "../src/server.js";
import { textPayload } from "./result.js";

const expectedTools = [
  "compile_actor_prompt",
  "create_sprite_session",
  "get_concept_candidate",
  "get_sprite_session",
  "get_studio_contract",
  "import_concept_candidate",
  "list_concept_candidates",
  "list_sprite_sessions",
];

const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();
const server = createActorStudioServer();
const client = new Client({
  name: "tileforge-actor-studio-smoke",
  version: "0.1.0",
});

try {
  await Promise.all([server.connect(serverTransport), client.connect(clientTransport)]);

  const tools = await client.listTools();
  const toolNames = tools.tools.map((tool) => tool.name).sort();
  if (JSON.stringify(toolNames) !== JSON.stringify(expectedTools)) {
    throw new Error(`Unexpected MCP tools: ${toolNames.join(", ")}`);
  }

  const contract = await client.callTool({
    name: "get_studio_contract",
    arguments: {},
  });
  const parsedContract = JSON.parse(textPayload(contract)) as {
    id?: string;
    approval?: { agentsMayApprove?: boolean };
  };
  if (
    parsedContract.id !== "tileforge-actor-32-v1" ||
    parsedContract.approval?.agentsMayApprove !== false
  ) {
    throw new Error("Contract identity or approval boundary changed.");
  }

  const compiled = await client.callTool({
    name: "compile_actor_prompt",
    arguments: {
      name: "Mirelight Pilgrim",
      kind: "mob",
      description: "A quiet reed-cloaked marsh pilgrim carrying a blue-green lantern.",
    },
  });
  if (!textPayload(compiled).includes("Only the user may approve final art")) {
    throw new Error("Compiled prompt omitted the human approval boundary.");
  }

  console.log(`MCP smoke passed (${toolNames.length} tools, contract locked).`);
} finally {
  await client.close();
  await server.close();
}
