import type { AnalysisRequest, OutputFile } from "@neherlab/treeknit-wasm";
import { isDeepEqual } from "remeda";

import type { TreeSource } from "../workspace/treeSource";
import {
  nextGeneration,
  type PersistenceChannel,
  type PersistenceMessage,
  RECORD_VERSION,
  type RecordStorage,
  type StoredRecord,
} from "./record";

export const SAVE_DELAY_MS = 500;

export interface WorkspaceSnapshot {
  request: AnalysisRequest;
  sources: TreeSource[];
}

export interface StoredWorkspace {
  sessionFile: string;
  sources: TreeSource[];
}

export type PersistenceProblemKind = "restore" | "enable" | "save" | "disable";

export interface PersistenceProblem {
  kind: PersistenceProblemKind;
  message: string;
}

export interface PersistenceState {
  enabled: boolean;
  problem: PersistenceProblem | null;
}

export interface PersistenceServices {
  storage: RecordStorage;
  channel: PersistenceChannel | null;
  requestFile: (request: AnalysisRequest) => Promise<OutputFile>;
}

interface Enabling {
  latest: WorkspaceSnapshot;
}

interface WrittenRecord {
  generation: number | null;
}

export class WorkspacePersistence {
  readonly #services: PersistenceServices;
  readonly #listeners = new Set<() => void>();
  #state: PersistenceState = { enabled: false, problem: null };
  #generation: number | null = null;
  #enabling: Enabling | undefined;
  #unsaved: WorkspaceSnapshot | undefined;
  #disablingEpoch: number | null = null;
  #epoch = 0;
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor(services: PersistenceServices) {
    this.#services = services;
    services.channel?.listen((message) => {
      this.#received(message);
    });
  }

  get state(): PersistenceState {
    return this.#state;
  }

  subscribe(listener: () => void): () => void {
    this.#listeners.add(listener);

    return () => {
      this.#listeners.delete(listener);
    };
  }

  async restore<T>(read: (stored: StoredWorkspace) => Promise<T>): Promise<T | null> {
    const epoch = this.#epoch;

    try {
      const record = await this.#services.storage.read();

      if (record?.kind !== "workspace") {
        return null;
      }

      const restored = await read({ sessionFile: record.sessionFile, sources: record.sources });
      const current = await this.#services.storage.read();

      if (epoch !== this.#epoch) {
        return null;
      }

      if (!isDeepEqual(current, record)) {
        return await this.restore(read);
      }

      this.#switchOn(record.generation);

      return restored;
    } catch (error) {
      this.#report("restore", error);

      return null;
    }
  }

  async enable(snapshot: WorkspaceSnapshot): Promise<void> {
    const epoch = this.#invalidate();
    const enabling: Enabling = { latest: snapshot };

    this.#enabling = enabling;

    try {
      await this.#enable(enabling, epoch);
    } catch (error) {
      if (epoch === this.#epoch) {
        this.#report("enable", error);
      }
    } finally {
      if (this.#enabling === enabling) {
        this.#enabling = undefined;
      }
    }
  }

  async disable(): Promise<void> {
    if (this.#generation === null && this.#enabling === undefined && this.#state.problem?.kind !== "disable") {
      return;
    }

    this.#enabling = undefined;
    this.#setProblem(null);

    const epoch = this.#invalidate();

    this.#disablingEpoch = epoch;

    try {
      const generation = await this.#write(epoch, (current) => ({
        kind: "off",
        version: RECORD_VERSION,
        generation: nextGeneration(current),
      }));

      if (generation !== null) {
        this.#services.channel?.post({ state: "off", generation });
      }

      if (generation !== null && epoch === this.#epoch) {
        this.#switchOff();
      }
    } catch (error) {
      if (epoch === this.#epoch) {
        this.#disablingEpoch = null;
        this.#report("disable", error);
        this.#resumeSaving();
      }
    }
  }

  changed(snapshot: WorkspaceSnapshot): void {
    if (this.#enabling !== undefined) {
      this.#enabling.latest = snapshot;

      return;
    }

    if (this.#generation === null) {
      return;
    }

    if (this.#disabling) {
      this.#unsaved = snapshot;

      return;
    }

    const epoch = this.#invalidate();

    this.#unsaved = snapshot;
    this.#timer = setTimeout(() => {
      this.#timer = undefined;
      void this.#save(snapshot, epoch);
    }, SAVE_DELAY_MS);
  }

  async saveNow(): Promise<void> {
    const snapshot = this.#unsaved;

    if (snapshot === undefined || this.#generation === null || this.#disabling) {
      return;
    }

    await this.#save(snapshot, this.#invalidate());
  }

  async #enable(enabling: Enabling, epoch: number): Promise<void> {
    const written = enabling.latest;
    const file = await this.#services.requestFile(written.request);

    if (epoch !== this.#epoch) {
      return;
    }

    if (enabling.latest !== written) {
      await this.#enable(enabling, epoch);

      return;
    }

    const generation = await this.#write(epoch, (current) => ({
      kind: "workspace",
      version: RECORD_VERSION,
      generation: nextGeneration(current),
      sessionFile: file.text,
      sources: written.sources,
    }));

    if (epoch !== this.#epoch || generation === null) {
      return;
    }

    this.#enabling = undefined;
    this.#setProblem(null);
    this.#switchOn(generation);
    this.#services.channel?.post({ state: "on", generation });

    if (enabling.latest !== written) {
      this.changed(enabling.latest);
    }
  }

  async #save(snapshot: WorkspaceSnapshot, epoch: number): Promise<void> {
    const generation = this.#generation;

    if (generation === null) {
      return;
    }

    try {
      const file = await this.#services.requestFile(snapshot.request);

      if (epoch !== this.#epoch || generation !== this.#generation) {
        return;
      }

      const outcome = { conflict: false };

      await this.#services.storage.update((current) => {
        if (epoch !== this.#epoch) {
          return "keep";
        }

        if (current?.generation !== generation) {
          outcome.conflict = true;

          return "keep";
        }

        return {
          kind: "workspace",
          version: RECORD_VERSION,
          generation,
          sessionFile: file.text,
          sources: snapshot.sources,
        };
      });

      if (outcome.conflict) {
        if (this.#generation === generation) {
          this.#switchOff();
        }

        return;
      }

      if (epoch === this.#epoch && this.#unsaved === snapshot) {
        this.#unsaved = undefined;

        if (this.#state.problem?.kind === "save") {
          this.#setProblem(null);
        }
      }
    } catch (error) {
      if (epoch === this.#epoch && generation === this.#generation) {
        this.#report("save", error);
      }
    }
  }

  async #write(epoch: number, record: (current: StoredRecord | undefined) => StoredRecord): Promise<number | null> {
    const written: WrittenRecord = { generation: null };

    await this.#services.storage.update((current) => {
      if (epoch !== this.#epoch) {
        return "keep";
      }

      const next = record(current);

      written.generation = next.generation;

      return next;
    });

    return written.generation;
  }

  get #disabling(): boolean {
    return this.#disablingEpoch === this.#epoch;
  }

  #resumeSaving(): void {
    const snapshot = this.#unsaved;

    if (snapshot !== undefined) {
      this.changed(snapshot);
    }
  }

  #received(message: PersistenceMessage): void {
    if (this.#generation !== null && message.generation !== this.#generation) {
      this.#switchOff();
    }
  }

  #switchOn(generation: number): void {
    this.#generation = generation;
    this.#setState({ ...this.#state, enabled: true });
  }

  #switchOff(): void {
    this.#invalidate();
    this.#unsaved = undefined;

    if (this.#generation !== null) {
      this.#generation = null;
      this.#setState({ ...this.#state, enabled: false });
    }
  }

  #report(kind: PersistenceProblemKind, cause: unknown): void {
    this.#setProblem({ kind, message: cause instanceof Error ? cause.message : String(cause) });
  }

  #setProblem(problem: PersistenceProblem | null): void {
    if (problem !== null || this.#state.problem !== null) {
      this.#setState({ ...this.#state, problem });
    }
  }

  #setState(state: PersistenceState): void {
    this.#state = state;

    for (const listener of this.#listeners) {
      listener();
    }
  }

  #invalidate(): number {
    clearTimeout(this.#timer);
    this.#timer = undefined;
    this.#epoch += 1;

    return this.#epoch;
  }
}
