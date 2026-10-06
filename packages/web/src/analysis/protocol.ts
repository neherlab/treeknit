import type * as wasm from "@neherlab/treeknit-wasm";
import type { AnalysisRequest, Progress, Session, Summary } from "@neherlab/treeknit-wasm";
import type { ProxyMarked } from "comlink";

export const WORKER_ONLY_EXPORTS = ["default", "initSync", "start", "setPanicSink", "Session"] as const;

export type StatelessApi = Omit<typeof wasm, (typeof WORKER_ONLY_EXPORTS)[number]>;

export type SessionApi = Omit<Session, "free" | typeof Symbol.dispose | "zip">;

export type StatelessArgs<Name extends keyof StatelessApi> = Parameters<StatelessApi[Name]>;

export type StatelessResult<Name extends keyof StatelessApi> = Promise<ReturnType<StatelessApi[Name]>>;

export type SessionArgs<Name extends keyof SessionApi> = Parameters<SessionApi[Name]>;

export type SessionResult<Name extends keyof SessionApi> = Promise<ReturnType<SessionApi[Name]>>;

export interface WorkerApi {
  readonly stateless: StatelessApi & ProxyMarked;
  readonly session: SessionApi & ProxyMarked;
  init(module: WebAssembly.Module): void;
  run(request: AnalysisRequest, onProgress: (progress: Progress) => Promise<void>): Summary;
  zip(): Uint8Array;
}
