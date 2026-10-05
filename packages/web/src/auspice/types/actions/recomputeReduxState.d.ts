import type { Dispatch } from "redux";

export declare function createStateFromQueryOrJSONs(params: {
  json: unknown;
  secondTreeDataset?: unknown;
  mainTreeName?: string;
  secondTreeName?: string;
  query: Readonly<Record<string, string>>;
  dispatch: Dispatch;
}): AuspiceCleanState;

interface AuspiceCleanState {
  metadata: unknown;
  tree: unknown;
  treeToo: unknown;
  controls: unknown;
  entropy: unknown;
  frequencies: unknown;
  narrative: unknown;
  measurements: unknown;
  query: unknown;
}
