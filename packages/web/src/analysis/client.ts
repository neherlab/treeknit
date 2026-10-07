import type { AnalysisRequest, Progress, Summary } from "@neherlab/treeknit-wasm";
import { type Endpoint, proxy, type Remote, wrap } from "comlink";
import { getErrorMessage } from "react-error-boundary";
import { doNothing } from "remeda";

import type { SessionApi, StatelessApi, WorkerApi } from "./protocol";

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

export interface CallOptions {
  signal?: AbortSignal | undefined;
}

export interface AnalysisClient {
  stateless<T>(operation: (api: Remote<StatelessApi>) => Promise<T>, options?: CallOptions): Promise<T>;
  inSession<T>(
    sessionId: number,
    operation: (session: Remote<SessionApi>) => Promise<T>,
    options?: CallOptions,
  ): Promise<T>;
  startRun(request: AnalysisRequest, onProgress: (progress: Progress) => void): RunHandle;
  cancel(): void;
  zip(sessionId: number, options?: CallOptions): Promise<Uint8Array>;
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
    this.#utility = this.#connectUtility();
  }

  async stateless<T>(operation: (api: Remote<StatelessApi>) => Promise<T>, options: CallOptions = {}): Promise<T> {
    return this.#onUtility(async (remote) => operation(remote.stateless), options.signal);
  }

  async inSession<T>(
    sessionId: number,
    operation: (session: Remote<SessionApi>) => Promise<T>,
    options: CallOptions = {},
  ): Promise<T> {
    return this.#onSession(sessionId, async (remote) => operation(remote.session), options.signal);
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

  async zip(sessionId: number, options: CallOptions = {}): Promise<Uint8Array> {
    return this.#onSession(sessionId, async (remote) => remote.zip(), options.signal);
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

      if (this.#run !== run) {
        run.connection.close();

        return;
      }

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

  async #onUtility<T>(operation: (remote: Remote<WorkerApi>) => Promise<T>, signal?: AbortSignal): Promise<T> {
    const connection = this.#utility;

    try {
      return await connection.call(operation, signal);
    } catch (error) {
      if (isWorkerFailure(error)) {
        this.#replaceUtility(connection, asError(error));
      }

      throw error;
    }
  }

  async #onSession<T>(
    sessionId: number,
    operation: (remote: Remote<WorkerApi>) => Promise<T>,
    signal?: AbortSignal,
  ): Promise<T> {
    const session = this.#session;

    if (session?.id !== sessionId) {
      throw new SessionUnavailableError();
    }

    try {
      return await session.connection.call(operation, signal);
    } catch (error) {
      if (isWorkerFailure(error)) {
        this.#lose(session);
      }

      throw error;
    }
  }

  #connectUtility(): Connection {
    const connection = this.#connect("treeknit-utility");

    connection.onCrash(() => {
      this.#replaceUtility(connection, new WorkerCrashError());
    });

    return connection;
  }

  #replaceUtility(connection: Connection, reason: Error): void {
    if (connection === this.#utility) {
      connection.close(reason);
      this.#utility = this.#connectUtility();
    }
  }

  #connect(name: string): Connection {
    return new Connection(() => this.#host.start(name), this.#compiled());
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
  readonly #worker: AnalysisWorker | undefined;
  readonly #failed: Promise<never>;
  readonly #ready: Promise<Remote<WorkerApi>>;
  readonly #fail: (error: Error) => void;
  readonly #queue: QueuedCall[] = [];
  #started = false;
  #closed = false;
  #draining = false;
  #stopped: Error | undefined;
  #crashListener: (() => void) | undefined;

  constructor(start: () => AnalysisWorker, module: Promise<WebAssembly.Module>) {
    const { promise, reject } = Promise.withResolvers<never>();

    this.#failed = promise;
    this.#fail = reject;
    promise.catch(doNothing());

    const opened = openWorker(start);

    this.#worker = opened instanceof WorkerStartError ? undefined : opened;
    this.#worker?.addEventListener("error", () => {
      this.#onError();
    });
    this.#ready = this.#init(opened, module);
    this.#ready.catch(doNothing());
  }

  async call<T>(operation: (remote: Remote<WorkerApi>) => Promise<T>, signal?: AbortSignal): Promise<T> {
    if (this.#stopped !== undefined) {
      throw this.#stopped;
    }

    signal?.throwIfAborted();

    const { promise, resolve, reject } = Promise.withResolvers<T>();

    const entry: QueuedCall = {
      start: async (remote) => {
        try {
          resolve(await Promise.race([operation(remote), this.#failed]));

          return undefined;
        } catch (error) {
          const failure = asError(error);

          reject(failure);

          return failure;
        }
      },
      reject,
      signal,
      abort: () => {
        this.#remove(entry);
        reject(signal?.reason);
      },
    };

    signal?.addEventListener("abort", entry.abort, { once: true });
    this.#queue.push(entry);
    void this.#drain();

    return promise;
  }

  onCrash(listener: () => void): void {
    this.#crashListener = listener;
  }

  close(reason: Error = new Error("The analysis worker was stopped.")): void {
    if (this.#closed) {
      return;
    }

    this.#closed = true;
    this.#stop(reason);
    this.#fail(reason);
    this.#worker?.terminate();
  }

  async #drain(): Promise<void> {
    if (this.#draining) {
      return;
    }

    this.#draining = true;

    try {
      const remote = await this.#ready;

      for (let entry = this.#next(); entry !== undefined; entry = this.#next()) {
        entry.signal?.removeEventListener("abort", entry.abort);
        const failure = await entry.start(remote);

        if (isWorkerFailure(failure)) {
          this.#stop(failure);
        }
      }
    } catch (error) {
      this.#stop(asError(error));
    } finally {
      this.#draining = false;
    }
  }

  #next(): QueuedCall | undefined {
    return this.#stopped === undefined ? this.#queue.shift() : undefined;
  }

  #remove(entry: QueuedCall): void {
    const index = this.#queue.indexOf(entry);

    if (index !== -1) {
      this.#queue.splice(index, 1);
    }
  }

  #stop(error: Error): void {
    if (this.#stopped !== undefined) {
      return;
    }

    this.#stopped = error;

    for (const entry of this.#queue.splice(0)) {
      entry.signal?.removeEventListener("abort", entry.abort);
      entry.reject(error);
    }
  }

  async #init(
    opened: AnalysisWorker | WorkerStartError,
    module: Promise<WebAssembly.Module>,
  ): Promise<Remote<WorkerApi>> {
    if (opened instanceof WorkerStartError) {
      throw opened;
    }

    const remote = wrap<WorkerApi>(opened);

    try {
      const compiled = await Promise.race([module, this.#failed]);

      await Promise.race([remote.init(compiled), this.#failed]);
    } catch (error) {
      throw error instanceof WorkerStartError ? error : new WorkerStartError(getErrorMessage(error) ?? String(error));
    }

    this.#started = true;

    return remote;
  }

  #onError(): void {
    if (this.#closed) {
      return;
    }

    if (this.#started) {
      const crash = new WorkerCrashError();

      this.#stop(crash);
      this.#fail(crash);
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

interface QueuedCall {
  start: (remote: Remote<WorkerApi>) => Promise<Error | undefined>;
  reject: (reason: Error) => void;
  signal: AbortSignal | undefined;
  abort: () => void;
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

function openWorker(start: () => AnalysisWorker): AnalysisWorker | WorkerStartError {
  try {
    return start();
  } catch (error) {
    return new WorkerStartError(getErrorMessage(error) ?? String(error));
  }
}

function isWorkerFailure(cause: unknown): cause is Error {
  return (
    cause instanceof WorkerStartError ||
    cause instanceof WorkerCrashError ||
    (cause instanceof Error && cause.name === "RuntimeError")
  );
}

function asError(cause: unknown): Error {
  return cause instanceof Error ? cause : new Error(String(cause));
}
