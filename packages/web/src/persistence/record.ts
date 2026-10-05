import * as z from "zod";

import { treeSourceSchema } from "../workspace/treeSource";

export const RECORD_VERSION = 1;

const generationSchema = z.int().min(0);

const versionSchema = z.literal(RECORD_VERSION);

export const storedRecordSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("workspace"),
    version: versionSchema,
    generation: generationSchema,
    sessionFile: z.string(),
    sources: z.array(treeSourceSchema),
  }),
  z.object({ kind: z.literal("off"), version: versionSchema, generation: generationSchema }),
]);

export type StoredRecord = z.output<typeof storedRecordSchema>;

export const persistenceMessageSchema = z.object({
  state: z.enum(["on", "off"]),
  generation: generationSchema,
});

export type PersistenceMessage = z.output<typeof persistenceMessageSchema>;

export type RecordUpdate = StoredRecord | "keep";

export interface RecordStorage {
  read(): Promise<StoredRecord | undefined>;
  update(updater: (current: StoredRecord | undefined) => RecordUpdate): Promise<void>;
}

export interface PersistenceChannel {
  post(message: PersistenceMessage): void;
  listen(listener: (message: PersistenceMessage) => void): void;
}

export const readableRecordSchema = storedRecordSchema.optional();

export class UnreadableRecordError extends Error {
  constructor(cause: z.ZodError) {
    super(`The workspace stored in this browser is not readable.\n${z.prettifyError(cause)}`, { cause });
  }
}

export function nextGeneration(current: StoredRecord | undefined): number {
  return (current?.generation ?? 0) + 1;
}
