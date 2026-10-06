import type { AnalysisRequest, Progress, Summary } from "@neherlab/treeknit-wasm";
import { expose } from "comlink";
import { describe, expect, test } from "vitest";

import { type AnalysisWorker, CANCELLED_MESSAGE, SessionUnavailableError, WorkerAnalysisClient } from "../client";
import type { WorkerApi } from "../protocol";

const REQUEST: AnalysisRequest = {
  trees: [
    { label: "ha", newick: "((A,B),(C,(D,X)));" },
    { label: "na", newick: "((A,(B,X)),(C,D));" },
  ],
};

const EMPTY_WASM_MODULE = new WebAssembly.Module(new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]));

const PROGRESS: Progress = { phase: "pairs", fraction: 0.5, round: 1, rounds: 1, pair: 1, pairs: 1 };

describe("analysis client", () => {
  test("answers stateless calls from the utility worker and compiles the module once", async () => {
    const host = new FakeHost({ utility: [fakeApi({ treeLabels: () => ["ha", "na_2"] })], job: [succeeding("one")] });
    const client = new WorkerAnalysisClient(host);

    const labels = await client.treeLabels(["ha.nwk", "na.nwk"], ["na"]);
    const outcome = await client.startRun(REQUEST, ignoreProgress).outcome;

    expect({ labels, outcome: outcome.status, compiled: host.compiled }).toStrictEqual({
      labels: ["ha", "na_2"],
      outcome: "succeeded",
      compiled: 1,
    });
    client.dispose();
  });

  test("serves session methods from the job worker of the latest successful run", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [succeeding("first"), succeeding("second")] });
    const client = new WorkerAnalysisClient(host);

    const first = await client.startRun(REQUEST, ignoreProgress).outcome;
    const firstLine = await client.commandLine(sessionId(first));
    const second = await client.startRun(REQUEST, ignoreProgress).outcome;
    const secondLine = await client.commandLine(sessionId(second));

    expect({
      lines: [firstLine, secondLine],
      firstGone: await client.commandLine(sessionId(first)).catch(errorName),
      terminated: host.terminatedNames(),
    }).toStrictEqual({
      lines: ["first", "second"],
      firstGone: "SessionUnavailableError",
      terminated: ["treeknit-job#0"],
    });
    client.dispose();
  });

  test("cancel settles the run as cancelled, stops only its job worker, and keeps the earlier results", async () => {
    const pending = new PendingRun();
    const host = new FakeHost({ utility: [fakeApi({})], job: [succeeding("kept"), pending.api] });
    const client = new WorkerAnalysisClient(host);
    const first = await client.startRun(REQUEST, ignoreProgress).outcome;

    const handle = client.startRun(REQUEST, ignoreProgress);

    await pending.started;
    client.cancel();
    const outcome = await handle.outcome;

    expect({
      outcome,
      terminated: host.terminatedNames(),
      kept: await client.commandLine(sessionId(first)),
    }).toStrictEqual({
      outcome: { status: "failed", kind: "cancelled", message: CANCELLED_MESSAGE },
      terminated: ["treeknit-job#1"],
      kept: "kept",
    });
    client.dispose();
  });

  test("ignores progress and the result that a cancelled run sends late", async () => {
    const pending = new PendingRun();
    const host = new FakeHost({ utility: [fakeApi({})], job: [succeeding("kept"), pending.api] });
    const client = new WorkerAnalysisClient(host);
    const first = await client.startRun(REQUEST, ignoreProgress).outcome;
    const progress: Progress[] = [];

    const handle = client.startRun(REQUEST, (event) => {
      progress.push(event);
    });

    await pending.started;
    client.cancel();
    pending.finish(summaryNamed("late"));
    await pending.progress(PROGRESS);

    expect({
      outcome: (await handle.outcome).status,
      progress,
      kept: await client.commandLine(sessionId(first)),
      lateSession: await client.commandLine(handle.runId).catch(errorName),
    }).toStrictEqual({ outcome: "failed", progress: [], kept: "kept", lateSession: "SessionUnavailableError" });
    client.dispose();
  });

  test("forwards the progress of the current run", async () => {
    const pending = new PendingRun();
    const host = new FakeHost({ utility: [fakeApi({})], job: [pending.api] });
    const client = new WorkerAnalysisClient(host);
    const progress: Progress[] = [];

    const handle = client.startRun(REQUEST, (event) => {
      progress.push(event);
    });

    await pending.started;
    await pending.progress(PROGRESS);
    pending.finish(summaryNamed("done"));

    expect({ outcome: (await handle.outcome).status, progress }).toStrictEqual({
      outcome: "succeeded",
      progress: [PROGRESS],
    });
    client.dispose();
  });

  test("classifies a ValidationError as invalid and any other error as internal", async () => {
    const host = new FakeHost({
      utility: [fakeApi({})],
      job: [failingRun("ValidationError", "trees[0].newick: unexpected end"), failingRun("Error", "pair failed")],
    });

    const client = new WorkerAnalysisClient(host);

    const invalid = await client.startRun(REQUEST, ignoreProgress).outcome;
    const internal = await client.startRun(REQUEST, ignoreProgress).outcome;

    expect({ invalid, internal, terminated: host.terminatedNames() }).toStrictEqual({
      invalid: { status: "failed", kind: "invalid", message: "trees[0].newick: unexpected end" },
      internal: { status: "failed", kind: "internal", message: "pair failed" },
      terminated: ["treeknit-job#0", "treeknit-job#1"],
    });
    client.dispose();
  });

  test("reports a job worker that fails to start as a start failure", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [undefined] });
    const client = new WorkerAnalysisClient(host);
    const handle = client.startRun(REQUEST, ignoreProgress);

    host.raiseError("treeknit-job#0");

    expect(await handle.outcome).toStrictEqual({
      status: "failed",
      kind: "start",
      message: "The analysis worker could not start.",
    });
    client.dispose();
  });

  test("settles a run as a start failure when the job worker cannot be constructed", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [] });
    const client = new WorkerAnalysisClient(host);

    host.refuseStart("treeknit-job");
    const handle = client.startRun(REQUEST, ignoreProgress);

    expect(await handle.outcome).toStrictEqual({
      status: "failed",
      kind: "start",
      message: "Worker construction is blocked.",
    });
    client.dispose();
  });

  test("a utility worker that cannot be constructed rejects the call with a start error and starts again", async () => {
    const host = new FakeHost({
      utility: [fakeApi({ version: () => ({ version: "0.5.0", repository: "r", releases: "r/releases" }) })],
      job: [],
    });

    host.refuseStart("treeknit-utility");
    const client = new WorkerAnalysisClient(host);
    const refused = await client.version().catch(errorName);
    const answered = await client.version();

    expect({ refused, answered }).toStrictEqual({
      refused: "WorkerStartError",
      answered: { version: "0.5.0", repository: "r", releases: "r/releases" },
    });
    client.dispose();
  });

  test("reports a module that fails to compile as a start failure and compiles again on the next run", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [succeeding("one"), succeeding("two")] });

    host.failCompile(new Error("Loading the analysis module failed with HTTP status 404."));
    const client = new WorkerAnalysisClient(host);
    const failed = await client.startRun(REQUEST, ignoreProgress).outcome;
    const succeeded = await client.startRun(REQUEST, ignoreProgress).outcome;

    expect({ failed, succeeded: succeeded.status, compiled: host.compiled }).toStrictEqual({
      failed: { status: "failed", kind: "start", message: "Loading the analysis module failed with HTTP status 404." },
      succeeded: "succeeded",
      compiled: 2,
    });
    client.dispose();
  });

  test("replaces the utility worker after a WebAssembly trap", async () => {
    const host = new FakeHost({
      utility: [
        fakeApi({ version: trap }),
        fakeApi({ version: () => ({ version: "0.5.0", repository: "r", releases: "r/releases" }) }),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);

    const trapped = await client.version().catch(errorName);
    const answered = await client.version();

    expect({ trapped, answered, terminated: host.terminatedNames() }).toStrictEqual({
      trapped: "RuntimeError",
      answered: { version: "0.5.0", repository: "r", releases: "r/releases" },
      terminated: ["treeknit-utility#0"],
    });
    client.dispose();
  });

  test("rejects the other calls of a replaced utility worker with the error that replaced it", async () => {
    const host = new FakeHost({
      utility: [fakeApi({ version: trap, palette: async () => Promise.withResolvers<never>().promise }), fakeApi({})],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);
    const waiting = client.palette().catch(errorName);
    const trapped = await client.version().catch(errorName);

    expect({ trapped, waiting: await waiting }).toStrictEqual({ trapped: "RuntimeError", waiting: "RuntimeError" });
    client.dispose();
  });

  test("replaces a utility worker that stops while idle, so the next call does not fail", async () => {
    const host = new FakeHost({
      utility: [
        fakeApi({ version: () => ({ version: "0.5.0", repository: "first", releases: "first/releases" }) }),
        fakeApi({ version: () => ({ version: "0.5.0", repository: "second", releases: "second/releases" }) }),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);
    const first = await client.version();

    host.raiseError("treeknit-utility#0");
    const second = await client.version();

    expect({ first: first.repository, second: second.repository, terminated: host.terminatedNames() }).toStrictEqual({
      first: "first",
      second: "second",
      terminated: ["treeknit-utility#0"],
    });
    client.dispose();
  });

  test("keeps the utility worker after an ordinary error", async () => {
    const host = new FakeHost({
      utility: [
        fakeApi({
          readSession: () => {
            throw new Error("missing field `trees`");
          },
        }),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);

    const message = await client
      .readSession("{}")
      .catch((error: unknown) => (error instanceof Error ? error.message : ""));

    expect({ message, terminated: host.terminatedNames() }).toStrictEqual({
      message: "missing field `trees`",
      terminated: [],
    });
    client.dispose();
  });

  test("clears the session and notifies listeners when the session worker traps", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [{ ...succeeding("x"), commandLine: trap }] });
    const client = new WorkerAnalysisClient(host);
    const lost: number[] = [];

    client.onSessionLost((id) => {
      lost.push(id);
    });
    const outcome = await client.startRun(REQUEST, ignoreProgress).outcome;
    const trapped = await client.commandLine(sessionId(outcome)).catch(errorName);
    const after = await client.commandLine(sessionId(outcome)).catch(errorName);

    expect({ trapped, after, lost, terminated: host.terminatedNames() }).toStrictEqual({
      trapped: "RuntimeError",
      after: "SessionUnavailableError",
      lost: [sessionId(outcome)],
      terminated: ["treeknit-job#0"],
    });
    client.dispose();
  });

  test("notifies listeners when the session worker stops on its own", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [succeeding("x")] });
    const client = new WorkerAnalysisClient(host);
    const lost: number[] = [];

    client.onSessionLost((id) => {
      lost.push(id);
    });
    const outcome = await client.startRun(REQUEST, ignoreProgress).outcome;

    host.raiseError("treeknit-job#0");
    const after = await client.summary(sessionId(outcome)).catch(errorName);

    expect({ lost, after }).toStrictEqual({ lost: [sessionId(outcome)], after: "SessionUnavailableError" });
    client.dispose();
  });

  test("rejects session calls before any run", async () => {
    const client = new WorkerAnalysisClient(new FakeHost({ utility: [fakeApi({})], job: [] }));

    await expect(client.files(1)).rejects.toBeInstanceOf(SessionUnavailableError);
    client.dispose();
  });
});

type FakeApi = {
  [Name in keyof WorkerApi]: (
    ...args: Parameters<WorkerApi[Name]>
  ) => ReturnType<WorkerApi[Name]> | Promise<ReturnType<WorkerApi[Name]>>;
};

class FakeHost {
  readonly #apis: Record<"utility" | "job", (FakeApi | undefined)[]>;
  readonly #ports = new Map<string, MessagePort>();
  readonly #terminated: string[] = [];
  readonly #refused = new Set<string>();
  #compileFailure: Error | undefined;
  compiled = 0;

  constructor(apis: Record<"utility" | "job", (FakeApi | undefined)[]>) {
    this.#apis = apis;
  }

  start(name: string): AnalysisWorker {
    if (this.#refused.delete(name)) {
      throw new TypeError("Worker construction is blocked.");
    }

    const role = name === "treeknit-utility" ? "utility" : "job";
    const index = [...this.#ports.keys()].filter((key) => key.startsWith(`${name}#`)).length;
    const key = `${name}#${String(index)}`;
    const { port1, port2 } = new MessageChannel();
    const api = this.#apis[role][index];

    if (api !== undefined) {
      expose(api, port2);
    }

    this.#ports.set(key, port1);

    return {
      postMessage: port1.postMessage.bind(port1),
      addEventListener: port1.addEventListener.bind(port1),
      removeEventListener: port1.removeEventListener.bind(port1),
      start: port1.start.bind(port1),
      terminate: () => {
        this.#terminated.push(key);
      },
    };
  }

  compile(): Promise<WebAssembly.Module> {
    this.compiled += 1;
    const failure = this.#compileFailure;

    this.#compileFailure = undefined;

    return failure === undefined ? Promise.resolve(EMPTY_WASM_MODULE) : Promise.reject(failure);
  }

  failCompile(error: Error): void {
    this.#compileFailure = error;
  }

  refuseStart(name: string): void {
    this.#refused.add(name);
  }

  raiseError(key: string): void {
    this.#ports.get(key)?.dispatchEvent(new Event("error"));
  }

  terminatedNames(): string[] {
    return [...this.#terminated];
  }
}

class PendingRun {
  readonly #started = Promise.withResolvers<undefined>();
  readonly #result = Promise.withResolvers<Summary>();
  #onProgress: ((progress: Progress) => Promise<void>) | undefined;
  readonly api: FakeApi;

  constructor() {
    this.api = fakeApi({
      run: async (_request, onProgress) => {
        this.#onProgress = onProgress;
        this.#started.resolve(undefined);

        return this.#result.promise;
      },
      commandLine: () => "pending",
    });
  }

  get started(): Promise<undefined> {
    return this.#started.promise;
  }

  async progress(progress: Progress): Promise<void> {
    await this.#onProgress?.(progress);
  }

  finish(summary: Summary): void {
    this.#result.resolve(summary);
  }
}

function succeeding(name: string): FakeApi {
  return fakeApi({
    run: () => summaryNamed(name),
    summary: () => summaryNamed(name),
    commandLine: () => name,
  });
}

function failingRun(name: string, message: string): FakeApi {
  return fakeApi({
    run: () => {
      throw Object.assign(new Error(message), { name });
    },
  });
}

function fakeApi(overrides: Partial<FakeApi>): FakeApi {
  const missing = () => {
    throw new Error("not faked");
  };

  return {
    init: () => undefined,
    defaultSettings: missing,
    settingsSchema: missing,
    inspectTree: missing,
    overlap: missing,
    validate: missing,
    readSession: missing,
    sessionFile: missing,
    treeLabels: missing,
    parseLaunch: missing,
    decodeTreeBytes: missing,
    examples: missing,
    launchKeys: missing,
    linkLimits: missing,
    launchPairs: missing,
    inlineSession: missing,
    applySettings: missing,
    version: missing,
    palette: missing,
    drawingRules: missing,
    run: missing,
    summary: missing,
    files: missing,
    fileText: missing,
    zip: missing,
    commandLine: missing,
    pairView: missing,
    auspiceView: missing,
    auspiceFiles: missing,
    argView: missing,
    constellation: missing,
    figure: missing,
    argFigure: missing,
    ...overrides,
  };
}

function trap(): never {
  throw new WebAssembly.RuntimeError("unreachable");
}

function summaryNamed(name: string): Summary {
  return {
    pairs: [],
    arg: null,
    noReassortment: false,
    diagnostics: [{ level: "info", message: name, time: "2026-01-01T00:00:00Z" }],
  };
}

function sessionId(outcome: { status: string; sessionId?: number }): number {
  if (outcome.sessionId === undefined) {
    throw new Error(`expected a successful run, got ${outcome.status}`);
  }

  return outcome.sessionId;
}

function errorName(error: unknown): string {
  return error instanceof Error ? error.name : "not an error";
}

function ignoreProgress(): undefined {
  return undefined;
}
