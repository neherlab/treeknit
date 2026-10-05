import wasmUrl from "@neherlab/treeknit-wasm/treeknit_wasm_bg.wasm?url";

import { type AnalysisClient, type WorkerHost, WorkerAnalysisClient } from "./client";

const WASM_FETCH_TIMEOUT_MS = 120_000;

const browserWorkerHost: WorkerHost = {
  start(name: string) {
    return new Worker(new URL("./worker.ts", import.meta.url), { type: "module", name });
  },
  async compile() {
    const response = await fetch(wasmUrl, { signal: AbortSignal.timeout(WASM_FETCH_TIMEOUT_MS) });

    if (!response.ok) {
      throw new Error(`Loading the analysis module failed with HTTP status ${String(response.status)}.`);
    }

    return WebAssembly.compile(await response.arrayBuffer());
  },
};

export function createAnalysisClient(): AnalysisClient {
  return new WorkerAnalysisClient(browserWorkerHost);
}
