import * as z from "zod";

export const treeSourceSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("file"), name: z.string() }),
  z.object({ kind: z.literal("paste") }),
  z.object({ kind: z.literal("example"), name: z.string(), id: z.string().optional() }),
  z.object({ kind: z.literal("session") }),
  z.object({ kind: z.literal("url"), url: z.string() }),
  z.object({ kind: z.literal("data") }),
  z.object({ kind: z.literal("message"), origin: z.string() }),
]);

export type TreeSource = z.output<typeof treeSourceSchema>;

export const SESSION_SOURCE: TreeSource = { kind: "session" };

export const PASTED_TREE_NAME = "tree";

export function sourceFileName(source: TreeSource): string {
  if (source.kind === "url") {
    return urlFileName(source.url);
  }

  return "name" in source ? source.name : PASTED_TREE_NAME;
}

export function sourceOrigin(source: TreeSource): string | undefined {
  if (source.kind === "url") {
    return URL.parse(source.url)?.host;
  }

  return source.kind === "message" ? source.origin : undefined;
}

function urlFileName(url: string): string {
  const segment = URL.parse(url)
    ?.pathname.split("/")
    .findLast((part) => part !== "");

  if (segment === undefined) {
    return PASTED_TREE_NAME;
  }

  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}
