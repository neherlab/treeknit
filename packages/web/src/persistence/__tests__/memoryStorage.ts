import type { PersistenceChannel, PersistenceMessage, RecordStorage, StoredRecord } from "../record";

export class MemoryStorage implements RecordStorage {
  record: StoredRecord | undefined;
  writes = 0;

  constructor(record?: StoredRecord) {
    this.record = record;
  }

  async read(): Promise<StoredRecord | undefined> {
    await Promise.resolve();

    return this.record;
  }

  async update(updater: (current: StoredRecord | undefined) => StoredRecord | undefined): Promise<void> {
    await Promise.resolve();

    const next = updater(this.record);

    if (next !== this.record) {
      this.writes += 1;
      this.record = next;
    }
  }
}

export class MemoryChannelHub {
  readonly #listeners = new Set<(message: PersistenceMessage) => void>();

  channel(): PersistenceChannel {
    const own = new Set<(message: PersistenceMessage) => void>();

    return {
      post: (message) => {
        for (const listener of this.#listeners) {
          if (!own.has(listener)) {
            listener(message);
          }
        }
      },
      listen: (listener) => {
        own.add(listener);
        this.#listeners.add(listener);
      },
      close: () => {
        for (const listener of own) {
          this.#listeners.delete(listener);
        }
      },
    };
  }
}
