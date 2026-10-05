import type { Analysis, AnalysisRequest, Settings } from "@neherlab/treeknit-wasm";
import { type Endpoint, type Remote, wrap } from "comlink";

import type { AnalysisApi } from "./worker";

export function startWorker(): AnalysisWorker {
  return new Worker(new URL("./worker.ts", import.meta.url), { type: "module", name: "treeknit-analysis" });
}

export class AnalysisClient {
  readonly #start: () => AnalysisWorker;
  #connection: Connection;

  constructor(start: () => AnalysisWorker) {
    this.#start = start;
    this.#connection = connect(start());
  }

  async analyze(request: AnalysisRequest): Promise<Analysis> {
    return this.#call((remote) => remote.analyze(request));
  }

  async defaultSettings(): Promise<Settings> {
    return this.#call((remote) => remote.defaultSettings());
  }

  dispose(): void {
    this.#connection.close();
  }

  async #call<T>(operation: (remote: Remote<AnalysisApi>) => Promise<T>): Promise<T> {
    const connection = this.#connection;

    try {
      return await Promise.race([operation(connection.remote), connection.failed]);
    } catch (error) {
      const trapped = error instanceof Error && error.name === "RuntimeError";

      if ((trapped || error instanceof WorkerStartError) && connection === this.#connection) {
        connection.close();
        this.#connection = connect(this.#start());
      }

      throw error;
    }
  }
}

export interface AnalysisWorker extends Endpoint {
  terminate(): void;
}

interface Connection {
  remote: Remote<AnalysisApi>;
  failed: Promise<never>;
  close(): void;
}

function connect(worker: AnalysisWorker): Connection {
  const { promise: failed, reject } = Promise.withResolvers<never>();
  failed.catch(ignoreUnobserved);
  worker.addEventListener("error", () => {
    reject(new WorkerStartError());
  });

  return {
    remote: wrap<AnalysisApi>(worker),
    failed,
    close() {
      worker.terminate();
    },
  };
}

function ignoreUnobserved(): undefined {
  return undefined;
}

class WorkerStartError extends Error {
  constructor() {
    super("The analysis worker could not start.");
    this.name = "WorkerStartError";
  }
}
