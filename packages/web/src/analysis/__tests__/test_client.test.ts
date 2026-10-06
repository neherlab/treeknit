import type { AnalysisRequest, AppVersion, Progress, Summary } from "@neherlab/treeknit-wasm";
import { expose, proxy } from "comlink";
import { describe, expect, test } from "vitest";

import { type AnalysisWorker, CANCELLED_MESSAGE, SessionUnavailableError, WorkerAnalysisClient } from "../client";
import type { SessionApi, StatelessApi } from "../protocol";

const REQUEST: AnalysisRequest = {
  trees: [
    { label: "ha", newick: "((A,B),(C,(D,X)));" },
    { label: "na", newick: "((A,(B,X)),(C,D));" },
  ],
};

const EMPTY_WASM_MODULE = new WebAssembly.Module(new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]));

const LABELS = ["a"];

const PROGRESS: Progress = { phase: "pairs", fraction: 0.5, round: 1, rounds: 1, pair: 1, pairs: 1 };

describe("analysis client", () => {
  test("answers stateless calls from the utility worker and compiles the module once", async () => {
    const host = new FakeHost({
      utility: [fakeApi({ stateless: { treeLabels: () => ["ha", "na_2"] } })],
      job: [succeeding("one")],
    });

    const client = new WorkerAnalysisClient(host);

    const labels = await client.stateless(async (api) => api.treeLabels(["ha.nwk", "na.nwk"], ["na"]));
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
    const firstLine = await commandLine(client, sessionId(first));
    const second = await client.startRun(REQUEST, ignoreProgress).outcome;
    const secondLine = await commandLine(client, sessionId(second));

    expect({
      lines: [firstLine, secondLine],
      firstGone: await commandLine(client, sessionId(first)).catch(errorName),
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
      kept: await commandLine(client, sessionId(first)),
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
      kept: await commandLine(client, sessionId(first)),
      lateSession: await commandLine(client, handle.runId).catch(errorName),
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
      utility: [
        fakeApi({ stateless: { version: () => ({ version: "0.5.0", repository: "r", releases: "r/releases" }) } }),
      ],
      job: [],
    });

    host.refuseStart("treeknit-utility");
    const client = new WorkerAnalysisClient(host);
    const refused = await version(client).catch(errorName);
    const answered = await version(client);

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
        fakeApi({ stateless: { version: trap } }),
        fakeApi({ stateless: { version: () => ({ version: "0.5.0", repository: "r", releases: "r/releases" }) } }),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);

    const trapped = await version(client).catch(errorName);
    const answered = await version(client);

    expect({ trapped, answered, terminated: host.terminatedNames() }).toStrictEqual({
      trapped: "RuntimeError",
      answered: { version: "0.5.0", repository: "r", releases: "r/releases" },
      terminated: ["treeknit-utility#0"],
    });
    client.dispose();
  });

  test("rejects a call queued behind a trapped call with the trap, and the worker never receives it", async () => {
    const received: string[] = [];

    const host = new FakeHost({
      utility: [
        fakeApi({
          stateless: {
            version: trap,
            treeLabels: () => {
              received.push("treeLabels");

              return LABELS;
            },
          },
        }),
        fakeApi({}),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);
    const trapped = version(client).catch((error: unknown) => error);
    const queued = labels(client).catch((error: unknown) => error);
    const rejected = await queued;

    await host.delivered("treeknit-utility#0");

    expect({ same: rejected === (await trapped), name: errorName(rejected), received }).toStrictEqual({
      same: true,
      name: "RuntimeError",
      received: [],
    });
    client.dispose();
  });

  test("passes the arguments of stateless and session calls to the worker", async () => {
    const calls: unknown[][] = [];

    const host = new FakeHost({
      utility: [fakeApi({ stateless: { treeLabels: (...args) => record(calls, args, LABELS) } })],
      job: [succeeding("x", { fileText: (...args) => record(calls, args, "text") })],
    });

    const client = new WorkerAnalysisClient(host);
    const answered = await client.stateless(async (api) => api.treeLabels(["ha.nwk"], ["na"]));
    const outcome = await client.startRun(REQUEST, ignoreProgress).outcome;
    const text = await client.inSession(sessionId(outcome), async (session) => session.fileText("MCCs.json"));

    expect({ answered, text, calls }).toStrictEqual({
      answered: LABELS,
      text: "text",
      calls: [[["ha.nwk"], ["na"]], ["MCCs.json"]],
    });
    client.dispose();
  });

  test("rejects a call aborted while queued with the abort reason, and the worker never receives it", async () => {
    const slow = new SlowCall();
    const received: string[] = [];
    const host = new FakeHost({ utility: [fakeApi({ stateless: slow.withLabels(received) })], job: [] });
    const client = new WorkerAnalysisClient(host);
    const controller = new AbortController();
    const reason = { stale: true };

    const first = version(client);
    await slow.started;
    const queued = labels(client, controller.signal);

    controller.abort(reason);
    const rejected = await queued.catch((error: unknown) => error);

    slow.finish();
    const answered = await first;

    await host.delivered("treeknit-utility#0");

    expect({ same: rejected === reason, first: answered.repository, received }).toStrictEqual({
      same: true,
      first: "slow",
      received: [],
    });
    client.dispose();
  });

  test("rejects a call whose signal is already aborted without reaching the worker", async () => {
    const received: string[] = [];
    const host = new FakeHost({ utility: [fakeApi({ stateless: new SlowCall().withLabels(received) })], job: [] });
    const client = new WorkerAnalysisClient(host);
    const reason = { stale: true };

    const rejected = await client
      .stateless(async (api) => api.treeLabels(["a.nwk"], []), { signal: AbortSignal.abort(reason) })
      .catch((error: unknown) => error);

    await host.delivered("treeknit-utility#0");

    expect({ same: rejected === reason, received }).toStrictEqual({ same: true, received: [] });
    client.dispose();
  });

  test("rejects a call aborted while the worker starts, and the worker never receives it", async () => {
    const gate = Promise.withResolvers<undefined>();
    const received: string[] = [];

    const host = new FakeHost({
      utility: [{ ...fakeApi({ stateless: new SlowCall().withLabels(received) }), init: async () => gate.promise }],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);
    const controller = new AbortController();
    const reason = { stale: true };
    const aborted = labels(client, controller.signal);

    controller.abort(reason);
    const rejected = await aborted.catch((error: unknown) => error);

    gate.resolve(undefined);
    const answered = await labels(client);

    expect({ same: rejected === reason, answered, received }).toStrictEqual({
      same: true,
      answered: LABELS,
      received: ["treeLabels"],
    });
    client.dispose();
  });

  test("keeps the result of a call aborted while it runs", async () => {
    const slow = new SlowCall();
    const host = new FakeHost({ utility: [fakeApi({ stateless: slow.withLabels([]) })], job: [] });
    const client = new WorkerAnalysisClient(host);
    const controller = new AbortController();

    const running = client.stateless(async (api) => api.version(), { signal: controller.signal });
    await slow.started;
    controller.abort({ stale: true });
    slow.finish();

    expect((await running).repository).toBe("slow");
    client.dispose();
  });

  test("rejects a queued call with the reason that stops the worker", async () => {
    const slow = new SlowCall();
    const received: string[] = [];
    const host = new FakeHost({ utility: [fakeApi({ stateless: slow.withLabels(received) })], job: [] });
    const client = new WorkerAnalysisClient(host);

    const running = version(client).catch(errorMessage);
    await slow.started;
    const queued = labels(client).catch(errorMessage);

    client.dispose();
    const stopped = { running: await running, queued: await queued };

    await host.delivered("treeknit-utility#0");

    expect({ ...stopped, received }).toStrictEqual({
      running: "The analysis worker was stopped.",
      queued: "The analysis worker was stopped.",
      received: [],
    });
  });

  test("replaces a utility worker that stops while idle, so the next call does not fail", async () => {
    const host = new FakeHost({
      utility: [
        fakeApi({
          stateless: { version: () => ({ version: "0.5.0", repository: "first", releases: "first/releases" }) },
        }),
        fakeApi({
          stateless: { version: () => ({ version: "0.5.0", repository: "second", releases: "second/releases" }) },
        }),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);
    const first = await version(client);

    host.raiseError("treeknit-utility#0");
    const second = await version(client);

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
          stateless: {
            readSession: () => {
              throw new Error("missing field `trees`");
            },
          },
        }),
      ],
      job: [],
    });

    const client = new WorkerAnalysisClient(host);

    const message = await client
      .stateless(async (api) => api.readSession("{}"))
      .catch((error: unknown) => (error instanceof Error ? error.message : ""));

    expect({ message, terminated: host.terminatedNames() }).toStrictEqual({
      message: "missing field `trees`",
      terminated: [],
    });
    client.dispose();
  });

  test("clears the session and notifies listeners when the session worker traps", async () => {
    const host = new FakeHost({ utility: [fakeApi({})], job: [succeeding("x", { commandLine: trap })] });
    const client = new WorkerAnalysisClient(host);
    const lost: number[] = [];

    client.onSessionLost((id) => {
      lost.push(id);
    });
    const outcome = await client.startRun(REQUEST, ignoreProgress).outcome;
    const trapped = await commandLine(client, sessionId(outcome)).catch(errorName);
    const after = await commandLine(client, sessionId(outcome)).catch(errorName);

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
    const after = await client.inSession(sessionId(outcome), async (session) => session.summary()).catch(errorName);

    expect({ lost, after }).toStrictEqual({ lost: [sessionId(outcome)], after: "SessionUnavailableError" });
    client.dispose();
  });

  test("rejects session calls before any run", async () => {
    const client = new WorkerAnalysisClient(new FakeHost({ utility: [fakeApi({})], job: [] }));

    await expect(client.inSession(1, async (session) => session.files())).rejects.toBeInstanceOf(
      SessionUnavailableError,
    );
    client.dispose();
  });
});

type Faked<Api> = {
  [Name in keyof Api]?: Api[Name] extends (...args: infer Args) => infer Result
    ? (...args: Args) => Result | Promise<Result>
    : never;
};

interface FakeApi {
  init: () => undefined | Promise<undefined>;
  run: (request: AnalysisRequest, onProgress: (progress: Progress) => Promise<void>) => Summary | Promise<Summary>;
  stateless: Faked<StatelessApi>;
  session: Faked<SessionApi>;
}

class FakeHost {
  readonly #apis: Record<"utility" | "job", (FakeApi | undefined)[]>;
  readonly #ports = new Map<string, MessagePort>();
  readonly #workerPorts = new Map<string, MessagePort>();
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
    this.#workerPorts.set(key, port2);

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

  async delivered(key: string): Promise<undefined> {
    const sender = this.#ports.get(key);
    const receiver = this.#workerPorts.get(key);

    if (sender === undefined || receiver === undefined) {
      throw new Error(`no worker ${key}`);
    }

    const { promise, resolve } = Promise.withResolvers<undefined>();

    receiver.addEventListener("message", function onMessage(event: MessageEvent<unknown>) {
      if (event.data === null) {
        receiver.removeEventListener("message", onMessage);
        resolve(undefined);
      }
    });
    sender.postMessage(null, []);

    return promise;
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
      session: { commandLine: () => "pending" },
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

class SlowCall {
  readonly #started = Promise.withResolvers<undefined>();
  readonly #result = Promise.withResolvers<undefined>();

  get started(): Promise<undefined> {
    return this.#started.promise;
  }

  finish(): void {
    this.#result.resolve(undefined);
  }

  withLabels(received: string[]): Faked<StatelessApi> {
    return {
      version: async () => {
        this.#started.resolve(undefined);
        await this.#result.promise;

        return { version: "0.5.0", repository: "slow", releases: "slow/releases" };
      },
      treeLabels: () => {
        received.push("treeLabels");

        return LABELS;
      },
    };
  }
}

function succeeding(name: string, session: Faked<SessionApi> = {}): FakeApi {
  return fakeApi({
    run: () => summaryNamed(name),
    session: { summary: () => summaryNamed(name), commandLine: () => name, ...session },
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
  return {
    init: () => undefined,
    run: () => {
      throw new Error("not faked");
    },
    ...overrides,
    stateless: proxy({ ...overrides.stateless }),
    session: proxy({ ...overrides.session }),
  };
}

async function commandLine(client: WorkerAnalysisClient, id: number): Promise<string> {
  return client.inSession(id, async (session) => session.commandLine());
}

async function labels(client: WorkerAnalysisClient, signal?: AbortSignal): Promise<string[]> {
  return client.stateless(async (api) => api.treeLabels(["a.nwk"], []), { signal });
}

async function version(client: WorkerAnalysisClient): Promise<AppVersion> {
  return client.stateless(async (api) => api.version());
}

function record<T>(calls: unknown[][], args: unknown[], result: T): T {
  calls.push(args);

  return result;
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

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "not an error";
}

function ignoreProgress(): undefined {
  return undefined;
}
