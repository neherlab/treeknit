import init, { analyze, defaultSettings } from "@neherlab/treeknit-wasm";
import type { Analysis, AnalysisRequest, Settings } from "@neherlab/treeknit-wasm";
import { expose } from "comlink";

const ready = init();

const api = {
  async analyze(request: AnalysisRequest): Promise<Analysis> {
    await ready;

    return analyze(request);
  },
  async defaultSettings(): Promise<Settings> {
    await ready;

    return defaultSettings();
  },
};

export type AnalysisApi = typeof api;

expose(api);
