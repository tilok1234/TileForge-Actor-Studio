export function textPayload(result: unknown): string {
  if (typeof result !== "object" || result === null) {
    throw new Error("MCP result was not an object.");
  }

  const content = Reflect.get(result, "content");
  if (!Array.isArray(content)) {
    throw new Error("MCP result did not contain a content array.");
  }

  const item = content.find(
    (entry): entry is { type: "text"; text: string } =>
      typeof entry === "object" &&
      entry !== null &&
      Reflect.get(entry, "type") === "text" &&
      typeof Reflect.get(entry, "text") === "string",
  );

  if (!item) {
    throw new Error("MCP result did not contain text.");
  }

  return item.text;
}
