import type { AnalysisRequest, Progress, Summary } from "@neherlab/treeknit-wasm";
import { type Endpoint, proxy, type Remote, wrap } from "comlink";

import type { SessionArgs, SessionResult, StatelessArgs, StatelessResult, WorkerApi } from "./protocol";

export const RESULTS_LOST_MESSAGE = "The results were lost because of an internal error. Run again.";

export const CANCELLED_MESSAGE = "Run cancelled.";

export type FailureKind = "invalid" | "internal" | "start" | "cancelled";

export type RunOutcome =
  | { status: "succeeded"; sessionId: number; summary: Summary }
  | { status: "failed"; kind: FailureKind; message: string };

export interface RunHandle {
  runId: number;
  outcome: Promise<RunOutcome>;
}

export interface AnalysisClient {
  defaultSettings(): StatelessResult<"defaultSettings">;
  settingsSchema(...args: StatelessArgs<"settingsSchema">): StatelessResult<"settingsSchema">;
  inspectTree(...args: StatelessArgs<"inspectTree">): StatelessResult<"inspectTree">;
  overlap(...args: StatelessArgs<"overlap">): StatelessResult<"overlap">;
  validate(...args: StatelessArgs<"validate">): StatelessResult<"validate">;
  readRequest(...args: StatelessArgs<"readRequest">): StatelessResult<"readRequest">;
  requestFile(...args: StatelessArgs<"requestFile">): StatelessResult<"requestFile">;
  treeLabels(...args: StatelessArgs<"treeLabels">): StatelessResult<"treeLabels">;
  version(): StatelessResult<"version">;
  palette(): StatelessResult<"palette">;
  startRun(request: AnalysisRequest, onProgress: (progress: Progress) => void): RunHandle;
  cancel(): void;
  summary(sessionId: number): SessionResult<"summary">;
  files(sessionId: number): SessionResult<"files">;
  fileText(sessionId: number, ...args: SessionArgs<"fileText">): SessionResult<"fileText">;
  zip(sessionId: number): SessionResult<"zip">;
  commandLine(sessionId: number): SessionResult<"commandLine">;
  pairView(sessionId: number, ...args: SessionArgs<"pairView">): SessionResult<"pairView">;
  argView(sessionId: number, ...args: SessionArgs<"argView">): SessionResult<"argView">;
  constellation(sessionId: number): SessionResult<"constellation">;
  figure(sessionId: number, ...args: SessionArgs<"figure">): SessionResult<"figure">;
  argFigure(sessionId: number, ...args: SessionArgs<"argFigure">): SessionResult<"argFigure">;
  onSessionLost(listener: (sessionId: number) => void): () => void;
  dispose(): void;
}

export interface AnalysisWorker extends Endpoint {
  terminate(): void;
}

export interface WorkerHost {
  start(name: string): AnalysisWorker;
  compile(): Promise<WebAssembly.Module>;
}

export class WorkerStartError extends Error {
  constructor(reason: string) {
    super(reason);
    this.name = "WorkerStartError";
  }
}

export class WorkerCrashError extends Error {
  constructor() {
    super("The analysis worker stopped unexpectedly.");
    this.name = "WorkerCrashError";
  }
}

export class SessionUnavailableError extends Error {
  constructor() {
    super("The results of this run are no longer available. Run again.");
    this.name = "SessionUnavailableError";
  }
}

export class WorkerAnalysisClient implements AnalysisClient {
  readonly #host: WorkerHost;
  readonly #lostListeners = new Set<(sessionId: number) => void>();
  #module: Promise<WebAssembly.Module> | undefined;
  #utility: Connection;
  #session: SessionWorker | undefined;
  #run: ActiveRun | undefined;
  #lastRunId = 0;

  constructor(host: WorkerHost) {
    this.#host = host;
    this.#utility = this.#connect("treeknit-utility");
  }

  async defaultSettings(): StatelessResult<"defaultSettings"> {
    return this.#stateless((remote) => remote.defaultSettings());
  }

  async settingsSchema(...args: StatelessArgs<"settingsSchema">): StatelessResult<"settingsSchema"> {
    return this.#stateless((remote) => remote.settingsSchema(...args));
  }

  async inspectTree(...args: StatelessArgs<"inspectTree">): StatelessResult<"inspectTree"> {
    return this.#stateless((remote) => remote.inspectTree(...args));
  }

  async overlap(...args: StatelessArgs<"overlap">): StatelessResult<"overlap"> {
    return this.#stateless((remote) => remote.overlap(...args));
  }

  async validate(...args: StatelessArgs<"validate">): StatelessResult<"validate"> {
    return this.#stateless((remote) => remote.validate(...args));
  }

  async readRequest(...args: StatelessArgs<"readRequest">): StatelessResult<"readRequest"> {
    return this.#stateless((remote) => remote.readRequest(...args));
  }

  async requestFile(...args: StatelessArgs<"requestFile">): StatelessResult<"requestFile"> {
    return this.#stateless((remote) => remote.requestFile(...args));
  }

  async treeLabels(...args: StatelessArgs<"treeLabels">): StatelessResult<"treeLabels"> {
    return this.#stateless((remote) => remote.treeLabels(...args));
  }

  async version(): StatelessResult<"version"> {
    return this.#stateless((remote) => remote.version());
  }

  async palette(): StatelessResult<"palette"> {
    return this.#stateless((remote) => remote.palette());
  }

  startRun(request: AnalysisRequest, onProgress: (progress: Progress) => void): RunHandle {
    this.cancel();
    this.#lastRunId += 1;

    const { promise, resolve } = Promise.withResolvers<RunOutcome>();
    const run: ActiveRun = { runId: this.#lastRunId, connection: this.#connect("treeknit-job"), settle: resolve };

    this.#run = run;
    void this.#execute(run, request, onProgress);

    return { runId: run.runId, outcome: promise };
  }

  cancel(): void {
    const run = this.#run;

    if (run === undefined) {
      return;
    }

    this.#run = undefined;
    run.connection.close();
    run.settle({ status: "failed", kind: "cancelled", message: CANCELLED_MESSAGE });
  }

  async summary(sessionId: number): SessionResult<"summary"> {
    return this.#inSession(sessionId, (remote) => remote.summary());
  }

  async files(sessionId: number): SessionResult<"files"> {
    return this.#inSession(sessionId, (remote) => remote.files());
  }

  async fileText(sessionId: number, ...args: SessionArgs<"fileText">): SessionResult<"fileText"> {
    return this.#inSession(sessionId, (remote) => remote.fileText(...args));
  }

  async zip(sessionId: number): SessionResult<"zip"> {
    return this.#inSession(sessionId, (remote) => remote.zip());
  }

  async commandLine(sessionId: number): SessionResult<"commandLine"> {
    return this.#inSession(sessionId, (remote) => remote.commandLine());
  }

  async pairView(sessionId: number, ...args: SessionArgs<"pairView">): SessionResult<"pairView"> {
    return this.#inSession(sessionId, (remote) => remote.pairView(...args));
  }

  async argView(sessionId: number, ...args: SessionArgs<"argView">): SessionResult<"argView"> {
    return this.#inSession(sessionId, (remote) => remote.argView(...args));
  }

  async constellation(sessionId: number): SessionResult<"constellation"> {
    return this.#inSession(sessionId, (remote) => remote.constellation());
  }

  async figure(sessionId: number, ...args: SessionArgs<"figure">): SessionResult<"figure"> {
    return this.#inSession(sessionId, (remote) => remote.figure(...args));
  }

  async argFigure(sessionId: number, ...args: SessionArgs<"argFigure">): SessionResult<"argFigure"> {
    return this.#inSession(sessionId, (remote) => remote.argFigure(...args));
  }

  onSessionLost(listener: (sessionId: number) => void): () => void {
    this.#lostListeners.add(listener);

    return () => {
      this.#lostListeners.delete(listener);
    };
  }

  dispose(): void {
    this.cancel();
    this.#utility.close();
    this.#session?.connection.close();
    this.#session = undefined;
    this.#lostListeners.clear();
  }

  async #execute(run: ActiveRun, request: AnalysisRequest, onProgress: (progress: Progress) => void): Promise<void> {
    const forward = proxy((progress: Progress): Promise<void> => {
      if (this.#run === run) {
        onProgress(progress);
      }

      return Promise.resolve();
    });

    try {
      const summary = await run.connection.call((remote) => remote.run(request, forward));

      this.#run = undefined;
      this.#promote(run);
      run.settle({ status: "succeeded", sessionId: run.runId, summary });
    } catch (error) {
      run.connection.close();

      if (this.#run === run) {
        this.#run = undefined;
        run.settle(runFailure(asError(error)));
      }
    }
  }

  #promote(run: ActiveRun): void {
    const previous = this.#session;
    const session: SessionWorker = { id: run.runId, connection: run.connection };

    this.#session = session;
    run.connection.onCrash(() => {
      this.#lose(session);
    });
    previous?.connection.close();
  }

  #lose(session: SessionWorker): void {
    if (this.#session !== session) {
      return;
    }

    this.#session = undefined;
    session.connection.close();

    for (const listener of this.#lostListeners) {
      listener(session.id);
    }
  }

  async #stateless<T>(operation: (remote: Remote<WorkerApi>) => Promise<T>): Promise<T> {
    const connection = this.#utility;

    try {
      return await connection.call(operation);
    } catch (error) {
      if (isWorkerFailure(error) && connection === this.#utility) {
        connection.close();
        this.#utility = this.#connect("treeknit-utility");
      }

      throw error;
    }
  }

  async #inSession<T>(sessionId: number, operation: (remote: Remote<WorkerApi>) => Promise<T>): Promise<T> {
    const session = this.#session;

    if (session?.id !== sessionId) {
      throw new SessionUnavailableError();
    }

    try {
      return await session.connection.call(operation);
    } catch (error) {
      if (isWorkerFailure(error)) {
        this.#lose(session);
      }

      throw error;
    }
  }

  #connect(name: string): Connection {
    return new Connection(this.#host.start(name), this.#compiled());
  }

  #compiled(): Promise<WebAssembly.Module> {
    if (this.#module === undefined) {
      const module = this.#host.compile();

      this.#module = module;
      module.catch(() => {
        if (this.#module === module) {
          this.#module = undefined;
        }
      });
    }

    return this.#module;
  }
}

class Connection {
  readonly #worker: AnalysisWorker;
  readonly #remote: Remote<WorkerApi>;
  readonly #failed: Promise<never>;
  readonly #ready: Promise<void>;
  readonly #fail: (error: Error) => void;
  #started = false;
  #closed = false;
  #crashListener: (() => void) | undefined;

  constructor(worker: AnalysisWorker, module: Promise<WebAssembly.Module>) {
    const { promise, reject } = Promise.withResolvers<never>();

    this.#worker = worker;
    this.#remote = wrap<WorkerApi>(worker);
    this.#failed = promise;
    this.#fail = reject;
    promise.catch(ignore);
    worker.addEventListener("error", () => {
      this.#onError();
    });
    this.#ready = this.#init(module);
    this.#ready.catch(ignore);
  }

  async call<T>(operation: (remote: Remote<WorkerApi>) => Promise<T>): Promise<T> {
    await this.#ready;

    try {
      return await Promise.race([operation(this.#remote), this.#failed]);
    } catch (error) {
      throw asError(error);
    }
  }

  onCrash(listener: () => void): void {
    this.#crashListener = listener;
  }

  close(): void {
    if (this.#closed) {
      return;
    }

    this.#closed = true;
    this.#fail(new Error("The analysis worker was stopped."));
    this.#worker.terminate();
  }

  async #init(module: Promise<WebAssembly.Module>): Promise<void> {
    try {
      const compiled = await Promise.race([module, this.#failed]);

      await Promise.race([this.#remote.init(compiled), this.#failed]);
    } catch (error) {
      throw error instanceof WorkerStartError ? error : new WorkerStartError(asError(error).message);
    }

    this.#started = true;
  }

  #onError(): void {
    if (this.#closed) {
      return;
    }

    if (this.#started) {
      this.#fail(new WorkerCrashError());
      this.#crashListener?.();
    } else {
      this.#fail(new WorkerStartError("The analysis worker could not start."));
    }
  }
}

interface ActiveRun {
  runId: number;
  connection: Connection;
  settle: (outcome: RunOutcome) => void;
}

interface SessionWorker {
  id: number;
  connection: Connection;
}

const FAILURE_KINDS: ReadonlyMap<string, FailureKind> = new Map([
  ["ValidationError", "invalid"],
  ["WorkerStartError", "start"],
]);

function runFailure({ name, message }: Error): RunOutcome {
  return { status: "failed", kind: FAILURE_KINDS.get(name) ?? "internal", message };
}

function isWorkerFailure(cause: unknown): boolean {
  return (
    cause instanceof WorkerStartError ||
    cause instanceof WorkerCrashError ||
    (cause instanceof Error && cause.name === "RuntimeError")
  );
}

function asError(cause: unknown): Error {
  return cause instanceof Error ? cause : new Error(String(cause));
}

function ignore(): undefined {
  return undefined;
}
