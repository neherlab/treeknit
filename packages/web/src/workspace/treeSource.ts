import type { UrlPlace } from "@neherlab/treeknit-wasm";
import { match } from "ts-pattern";
import * as z from "zod";

const urlSourceSchema = z.object({ kind: z.literal("url"), url: z.string(), name: z.string(), host: z.string() });

const otherSourceSchemas = [
  z.object({ kind: z.literal("file"), name: z.string() }),
  z.object({ kind: z.literal("paste") }),
  z.object({ kind: z.literal("example"), name: z.string(), id: z.string().optional() }),
  z.object({ kind: z.literal("session") }),
  z.object({ kind: z.literal("data") }),
  z.object({ kind: z.literal("message"), origin: z.string() }),
] as const;

export const treeSourceSchema = z.discriminatedUnion("kind", [...otherSourceSchemas, urlSourceSchema]);

export const storedTreeSourceSchema = z.discriminatedUnion("kind", [
  ...otherSourceSchemas,
  urlSourceSchema.partial({ name: true, host: true }),
]);

export type TreeSource = z.output<typeof treeSourceSchema>;

export type StoredTreeSource = z.output<typeof storedTreeSourceSchema>;

export const SESSION_SOURCE: TreeSource = { kind: "session" };

export function urlSource(url: string, { host, fileName }: UrlPlace): TreeSource {
  return { kind: "url", url, name: fileName, host };
}

export async function restoredSources(
  sources: readonly StoredTreeSource[],
  urlPlace: (url: string) => Promise<UrlPlace>,
): Promise<TreeSource[]> {
  return Promise.all(
    sources.map(async (source) => (source.kind === "url" ? urlSource(source.url, await urlPlace(source.url)) : source)),
  );
}

export function sourceFileName(source: TreeSource): string {
  return "name" in source ? source.name : "";
}

export function sourceOrigin(source: TreeSource): string | undefined {
  return match(source)
    .with({ kind: "url" }, ({ host }) => host)
    .with({ kind: "message" }, ({ origin }) => origin)
    .otherwise(() => undefined);
}
