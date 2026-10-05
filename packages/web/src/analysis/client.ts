import type {
  AnalysisRequest,
  AppVersion,
  ArgView,
  ConstellationTable,
  FigureOptions,
  FileEntry,
  OutputFile,
  Overlap,
  PairView,
  Palette,
  Progress,
  Scale,
  Settings,
  SettingsSchema,
  Summary,
  TreeInspection,
  TreeText,
  ValidationError,
  Version,
} from "@neherlab/treeknit-wasm";
import { type Endpoint, proxy, type Remote, wrap } from "comlink";

import type { WorkerApi } from "./protocol";

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
  defaultSettings(): Promise<Settings>;
  settingsSchema(k: number, settings: Settings): Promise<SettingsSchema>;
  inspectTree(label: string, text: string): Promise<TreeInspection>;
  overlap(trees: TreeText[]): Promise<Overlap>;
  validate(request: AnalysisRequest): Promise<ValidationError[]>;
  readRequest(text: string): Promise<AnalysisRequest>;
  requestFile(request: AnalysisRequest): Promise<OutputFile>;
  treeLabels(fileNames: string[], existingLabels: string[]): Promise<string[]>;
  version(): Promise<AppVersion>;
  palette(): Promise<Palette>;
  startRun(request: AnalysisRequest, onProgress: (progress: Progress) => void): RunHandle;
  cancel(): void;
  summary(sessionId: number): Promise<Summary>;
  files(sessionId: number): Promise<FileEntry[]>;
  fileText(sessionId: number, path: string): Promise<string>;
  zip(sessionId: number): Promise<Uint8Array>;
  commandLine(sessionId: number): Promise<string>;
  pairView(sessionId: number, pair: number, version: Version, scale: Scale): Promise<PairView>;
  argView(sessionId: number, scale: Scale): Promise<ArgView | undefined>;
  constellation(sessionId: number): Promise<ConstellationTable>;
  figure(sessionId: number, pair: number, version: Version, options: FigureOptions): Promise<string>;
  argFigure(sessionId: number, options: FigureOptions): Promise<string>;
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

  async defaultSettings(): Promise<Settings> {
    return this.#stateless((remote) => remote.defaultSettings());
  }

  async settingsSchema(k: number, settings: Settings): Promise<SettingsSchema> {
    return this.#stateless((remote) => remote.settingsSchema(k, settings));
  }

  async inspectTree(label: string, text: string): Promise<TreeInspection> {
    return this.#stateless((remote) => remote.inspectTree(label, text));
  }

  async overlap(trees: TreeText[]): Promise<Overlap> {
    return this.#stateless((remote) => remote.overlap(trees));
  }

  async validate(request: AnalysisRequest): Promise<ValidationError[]> {
    return this.#stateless((remote) => remote.validate(request));
  }

  async readRequest(text: string): Promise<AnalysisRequest> {
    return this.#stateless((remote) => remote.readRequest(text));
  }

  async requestFile(request: AnalysisRequest): Promise<OutputFile> {
    return this.#stateless((remote) => remote.requestFile(request));
  }

  async treeLabels(fileNames: string[], existingLabels: string[]): Promise<string[]> {
    return this.#stateless((remote) => remote.treeLabels(fileNames, existingLabels));
  }

  async version(): Promise<AppVersion> {
    return this.#stateless((remote) => remote.version());
  }

  async palette(): Promise<Palette> {
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

  async summary(sessionId: number): Promise<Summary> {
    return this.#inSession(sessionId, (remote) => remote.summary());
  }

  async files(sessionId: number): Promise<FileEntry[]> {
    return this.#inSession(sessionId, (remote) => remote.files());
  }

  async fileText(sessionId: number, path: string): Promise<string> {
    return this.#inSession(sessionId, (remote) => remote.fileText(path));
  }

  async zip(sessionId: number): Promise<Uint8Array> {
    return this.#inSession(sessionId, (remote) => remote.zip());
  }

  async commandLine(sessionId: number): Promise<string> {
    return this.#inSession(sessionId, (remote) => remote.commandLine());
  }

  async pairView(sessionId: number, pair: number, version: Version, scale: Scale): Promise<PairView> {
    return this.#inSession(sessionId, (remote) => remote.pairView(pair, version, scale));
  }

  async argView(sessionId: number, scale: Scale): Promise<ArgView | undefined> {
    return this.#inSession(sessionId, (remote) => remote.argView(scale));
  }

  async constellation(sessionId: number): Promise<ConstellationTable> {
    return this.#inSession(sessionId, (remote) => remote.constellation());
  }

  async figure(sessionId: number, pair: number, version: Version, options: FigureOptions): Promise<string> {
    return this.#inSession(sessionId, (remote) => remote.figure(pair, version, options));
  }

  async argFigure(sessionId: number, options: FigureOptions): Promise<string> {
    return this.#inSession(sessionId, (remote) => remote.argFigure(options));
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
