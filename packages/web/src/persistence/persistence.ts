import type { AnalysisRequest, OutputFile } from "@neherlab/treeknit-wasm";

import type { TreeSource } from "../workspace/treeSource";
import { nextGeneration, type PersistenceChannel, type PersistenceMessage, type RecordStorage } from "./record";

export const SAVE_DELAY_MS = 500;

export interface WorkspaceSnapshot {
  request: AnalysisRequest;
  sources: TreeSource[];
}

export interface StoredWorkspace {
  sessionFile: string;
  sources: TreeSource[];
}

export interface PersistenceServices {
  storage: RecordStorage;
  channel: PersistenceChannel | null;
  requestFile: (request: AnalysisRequest) => Promise<OutputFile>;
  onSwitchChange: (enabled: boolean) => void;
}

export class WorkspacePersistence {
  readonly #services: PersistenceServices;
  #generation: number | null = null;
  #epoch = 0;
  #timer: ReturnType<typeof setTimeout> | undefined;

  constructor(services: PersistenceServices) {
    this.#services = services;
    services.channel?.listen((message) => {
      this.#received(message);
    });
  }

  get enabled(): boolean {
    return this.#generation !== null;
  }

  async restore(): Promise<StoredWorkspace | null> {
    const record = await this.#services.storage.read();

    if (record?.kind !== "workspace") {
      return null;
    }

    this.#switchOn(record.generation);

    return { sessionFile: record.sessionFile, sources: record.sources };
  }

  async enable(snapshot: WorkspaceSnapshot): Promise<void> {
    const epoch = this.#invalidate();
    const file = await this.#services.requestFile(snapshot.request);

    if (epoch !== this.#epoch) {
      return;
    }

    const written = { generation: 0 };

    await this.#services.storage.update((current) => {
      written.generation = nextGeneration(current);

      return {
        kind: "workspace",
        generation: written.generation,
        sessionFile: file.text,
        sources: snapshot.sources,
      };
    });
    this.#switchOn(written.generation);
    this.#services.channel?.post({ state: "on", generation: written.generation });
  }

  async disable(): Promise<void> {
    if (this.#generation === null) {
      return;
    }

    this.#switchOff();

    const written = { generation: 0 };

    await this.#services.storage.update((current) => {
      written.generation = nextGeneration(current);

      return { kind: "off", generation: written.generation };
    });
    this.#services.channel?.post({ state: "off", generation: written.generation });
  }

  changed(snapshot: WorkspaceSnapshot): void {
    if (this.#generation === null) {
      return;
    }

    const epoch = this.#invalidate();

    this.#timer = setTimeout(() => {
      this.#timer = undefined;
      void this.#save(snapshot, epoch);
    }, SAVE_DELAY_MS);
  }

  dispose(): void {
    this.#invalidate();
    this.#services.channel?.close();
  }

  async #save(snapshot: WorkspaceSnapshot, epoch: number): Promise<void> {
    const file = await this.#services.requestFile(snapshot.request).catch(() => undefined);
    const generation = this.#generation;

    if (file === undefined || epoch !== this.#epoch || generation === null) {
      return;
    }

    const outcome = { conflict: false };

    await this.#services.storage.update((current) => {
      if (epoch !== this.#epoch) {
        return current;
      }

      if (current?.generation !== generation) {
        outcome.conflict = true;

        return current;
      }

      return { kind: "workspace", generation, sessionFile: file.text, sources: snapshot.sources };
    });

    if (outcome.conflict && this.#generation === generation) {
      this.#switchOff();
    }
  }

  #received(message: PersistenceMessage): void {
    if (this.#generation !== null && message.generation !== this.#generation) {
      this.#switchOff();
    }
  }

  #switchOn(generation: number): void {
    this.#generation = generation;
    this.#services.onSwitchChange(true);
  }

  #switchOff(): void {
    this.#invalidate();

    if (this.#generation !== null) {
      this.#generation = null;
      this.#services.onSwitchChange(false);
    }
  }

  #invalidate(): number {
    clearTimeout(this.#timer);
    this.#timer = undefined;
    this.#epoch += 1;

    return this.#epoch;
  }
}
