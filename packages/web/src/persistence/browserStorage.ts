import { createStore, get, update } from "idb-keyval";
import * as z from "zod";

import {
  type PersistenceChannel,
  type PersistenceMessage,
  persistenceMessageSchema,
  type RecordStorage,
  storedRecordSchema,
} from "./record";

const DATABASE = "treeknit";

const OBJECT_STORE = "workspace";

const RECORD_KEY = "workspace";

const CHANNEL = "treeknit-workspace";

export function indexedDbStorage(): RecordStorage {
  const store = createStore(DATABASE, OBJECT_STORE);

  return {
    async read() {
      const stored = await get<unknown>(RECORD_KEY, store);

      if (stored === undefined) {
        return undefined;
      }

      const record = storedRecordSchema.safeParse(stored);

      if (!record.success) {
        throw new Error(`The workspace stored in this browser is not readable.\n${z.prettifyError(record.error)}`);
      }

      return record.data;
    },
    async update(updater) {
      await update<unknown>(RECORD_KEY, (current) => updater(storedRecordSchema.safeParse(current).data), store);
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
