import type * as wasm from "@neherlab/treeknit-wasm";
import type { AnalysisRequest, Progress, Session, Summary } from "@neherlab/treeknit-wasm";

export type StatelessApi = Pick<
  typeof wasm,
  | "defaultSettings"
  | "settingsSchema"
  | "inspectTree"
  | "overlap"
  | "validate"
  | "readSession"
  | "sessionFile"
  | "treeLabels"
  | "parseLaunch"
  | "decodeTreeBytes"
  | "examples"
  | "launchKeys"
  | "linkLimits"
  | "launchPairs"
  | "inlineSession"
  | "applySettings"
  | "version"
  | "palette"
  | "drawingRules"
>;

export type SessionApi = Omit<Session, "free" | typeof Symbol.dispose>;

export type StatelessArgs<Name extends keyof StatelessApi> = Parameters<StatelessApi[Name]>;

export type StatelessResult<Name extends keyof StatelessApi> = Promise<ReturnType<StatelessApi[Name]>>;

export type SessionArgs<Name extends keyof SessionApi> = Parameters<SessionApi[Name]>;

export type SessionResult<Name extends keyof SessionApi> = Promise<ReturnType<SessionApi[Name]>>;

export interface WorkerApi extends StatelessApi, SessionApi {
  init(module: WebAssembly.Module): void;
  run(request: AnalysisRequest, onProgress: (progress: Progress) => Promise<void>): Summary;
}
