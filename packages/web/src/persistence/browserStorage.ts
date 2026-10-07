import { createStore, get, update } from "idb-keyval";

import {
  type PersistenceChannel,
  type PersistenceMessage,
  persistenceMessageSchema,
  readableRecordSchema,
  type RecordStorage,
  UnreadableRecordError,
} from "./record";

const DATABASE = "treeknit";

const OBJECT_STORE = "workspace";

const RECORD_KEY = "workspace";

const CHANNEL = "treeknit-workspace";

export function indexedDbStorage(): RecordStorage {
  const store = createStore(DATABASE, OBJECT_STORE);

  return {
    async read() {
      const record = readableRecordSchema.safeParse(await get<unknown>(RECORD_KEY, store));

      if (!record.success) {
        throw new UnreadableRecordError(record.error);
      }

      return record.data;
    },
    async update(updater) {
      await update<unknown>(
        RECORD_KEY,
        (stored) => {
          const next = updater(readableRecordSchema.safeParse(stored).data);

          return next === "keep" ? stored : next;
        },
        store,
      );
    },
  };
}

export function broadcastChannel(): PersistenceChannel | null {
  if (!("BroadcastChannel" in globalThis)) {
    return null;
  }

  const channel = new BroadcastChannel(CHANNEL);

  return {
    post(message: PersistenceMessage) {
      // oxlint-disable-next-line unicorn/require-post-message-target-origin -- BroadcastChannel.postMessage takes no target origin; the channel is same-origin
      channel.postMessage(message);
    },
    listen(listener: (message: PersistenceMessage) => void) {
      channel.addEventListener("message", (event) => {
        const message = persistenceMessageSchema.safeParse(event.data);

        if (message.success) {
          listener(message.data);
        }
      });
    },
  };
}
