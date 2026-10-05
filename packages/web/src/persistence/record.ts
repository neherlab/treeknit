import * as z from "zod";

import { treeSourceSchema } from "../workspace/treeSource";

const generationSchema = z.int().min(0);

export const storedRecordSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("workspace"),
    generation: generationSchema,
    sessionFile: z.string(),
    sources: z.array(treeSourceSchema),
  }),
  z.object({ kind: z.literal("off"), generation: generationSchema }),
]);

export type StoredRecord = z.output<typeof storedRecordSchema>;

export const persistenceMessageSchema = z.object({
  state: z.enum(["on", "off"]),
  generation: generationSchema,
});

export type PersistenceMessage = z.output<typeof persistenceMessageSchema>;

export interface RecordStorage {
  read(): Promise<StoredRecord | undefined>;
  update(updater: (current: StoredRecord | undefined) => StoredRecord | undefined): Promise<void>;
}

export interface PersistenceChannel {
  post(message: PersistenceMessage): void;
  listen(listener: (message: PersistenceMessage) => void): void;
  close(): void;
}

export function nextGeneration(current: StoredRecord | undefined): number {
  return (current?.generation ?? 0) + 1;
}
