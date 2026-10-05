import type { PersistenceChannel, PersistenceMessage, RecordStorage, StoredRecord } from "../record";

export class MemoryStorage implements RecordStorage {
  record: StoredRecord | undefined;
  writes = 0;
  #readFailure: Error | undefined;
  #updateFailure: Error | undefined;
  #held: PromiseWithResolvers<undefined> | undefined;
  #started: PromiseWithResolvers<undefined> | undefined;

  constructor(record?: StoredRecord) {
    this.record = record;
  }

  failNextRead(error: Error): void {
    this.#readFailure = error;
  }

  failNextUpdate(error: Error): void {
    this.#updateFailure = error;
  }

  holdUpdate(): void {
    this.#held = Promise.withResolvers<undefined>();
    this.#started = Promise.withResolvers<undefined>();
  }

  async updateStarted(): Promise<void> {
    await this.#started?.promise;
  }

  releaseUpdate(): void {
    this.#held?.resolve(undefined);
    this.#held = undefined;
  }

  async read(): Promise<StoredRecord | undefined> {
    await Promise.resolve();

    const failure = this.#readFailure;

    this.#readFailure = undefined;

    if (failure !== undefined) {
      throw failure;
    }

    return this.record;
  }

  async update(updater: (current: StoredRecord | undefined) => StoredRecord | undefined): Promise<void> {
    const held = this.#held;

    this.#started?.resolve(undefined);
    this.#started = undefined;
    await Promise.resolve();
    await held?.promise;

    const failure = this.#updateFailure;

    this.#updateFailure = undefined;

    if (failure !== undefined) {
      throw failure;
    }

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
    };
  }
}
