#!/usr/bin/env node

import { createMcpExpressApp } from "@modelcontextprotocol/sdk/server/express.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import type { Request, Response } from "express";
import { createActorStudioServer } from "./server.js";

function argument(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

async function startStdio(): Promise<void> {
  const server = createActorStudioServer();
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("TileForge Actor Studio MCP listening on stdio.");
}

async function startHttp(): Promise<void> {
  const requestedPort = Number(argument("--port") ?? process.env.TFAS_MCP_PORT ?? "7331");
  if (!Number.isInteger(requestedPort) || requestedPort < 1024 || requestedPort > 65535) {
    throw new Error("MCP port must be an integer between 1024 and 65535.");
  }

  const app = createMcpExpressApp({ host: "127.0.0.1" });

  app.get("/health", (_request: Request, response: Response) => {
    response.json({
      ok: true,
      service: "tileforge-actor-studio",
      transport: "streamable-http",
    });
  });

  app.post("/mcp", async (request: Request, response: Response) => {
    const server = createActorStudioServer();
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true,
    });

    response.on("close", () => {
      void transport.close();
      void server.close();
    });

    try {
      await server.connect(transport);
      await transport.handleRequest(request, response, request.body);
    } catch (error) {
      console.error("MCP request failed:", error);
      if (!response.headersSent) {
        response.status(500).json({
          jsonrpc: "2.0",
          error: { code: -32603, message: "Internal server error" },
          id: null,
        });
      }
    }
  });

  app.get("/mcp", (_request: Request, response: Response) => {
    response.status(405).json({
      jsonrpc: "2.0",
      error: { code: -32000, message: "Method not allowed." },
      id: null,
    });
  });

  app.delete("/mcp", (_request: Request, response: Response) => {
    response.status(405).json({
      jsonrpc: "2.0",
      error: { code: -32000, message: "Method not allowed." },
      id: null,
    });
  });

  app.listen(requestedPort, "127.0.0.1", (error?: Error) => {
    if (error) {
      console.error(error);
      process.exit(1);
    }
    console.error(
      `TileForge Actor Studio MCP listening on http://127.0.0.1:${requestedPort}/mcp`,
    );
  });
}

const transport = argument("--transport") ?? "stdio";
if (transport === "stdio") {
  await startStdio();
} else if (transport === "http") {
  await startHttp();
} else {
  throw new Error(`Unsupported transport: ${transport}`);
}
