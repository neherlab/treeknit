import * as z from "zod";

export const treeSourceSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("file"), name: z.string() }),
  z.object({ kind: z.literal("paste") }),
  z.object({ kind: z.literal("example"), name: z.string() }),
  z.object({ kind: z.literal("session") }),
]);

export type TreeSource = z.output<typeof treeSourceSchema>;

export const SESSION_SOURCE: TreeSource = { kind: "session" };

export const PASTED_TREE_NAME = "tree";

export function sourceFileName(source: TreeSource): string {
  return "name" in source ? source.name : PASTED_TREE_NAME;
}
