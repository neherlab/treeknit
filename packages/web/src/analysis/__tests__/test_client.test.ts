import type { Analysis, AnalysisRequest } from "@neherlab/treeknit-wasm";
import { expose } from "comlink";
import { describe, expect, test } from "vitest";

import { AnalysisClient, type AnalysisWorker } from "../client";
import type { AnalysisApi } from "../worker";

const REQUEST: AnalysisRequest = {
  trees: [
    { label: "ha", newick: "((A,B),(C,(D,X)));" },
    { label: "na", newick: "((A,(B,X)),(C,D));" },
  ],
};

const ANALYSIS: Analysis = {
  pairs: [{ trees: ["ha", "na"], mccs: [["X"], ["A", "B", "C", "D"]] }],
  arg: { status: "built", reassortments: 1 },
  files: [{ name: "MCCs.dat", mediaType: "text/plain", text: "X\nA,B,C,D\n" }],
};

describe("analysis client", () => {
  test("returns the analysis of the worker", async () => {
    const workers = new FakeWorkers([answering(ANALYSIS)]);
    const client = new AnalysisClient(workers.start);

    await expect(client.analyze(REQUEST)).resolves.toStrictEqual(ANALYSIS);
    client.dispose();
  });

  test("passes an analysis error on and keeps the worker", async () => {
    const workers = new FakeWorkers([failing(new Error("need at least two trees")), answering(ANALYSIS)]);
    const client = new AnalysisClient(workers.start);

    await expect(client.analyze(REQUEST)).rejects.toThrow("need at least two trees");
    expect(workers.started).toBe(1);
    client.dispose();
  });

  test("replaces the worker after a WebAssembly trap", async () => {
    const workers = new FakeWorkers([failing(new WebAssembly.RuntimeError("unreachable")), answering(ANALYSIS)]);
    const client = new AnalysisClient(workers.start);

    await expect(client.analyze(REQUEST)).rejects.toMatchObject({ name: "RuntimeError", message: "unreachable" });
    await expect(client.analyze(REQUEST)).resolves.toStrictEqual(ANALYSIS);
    expect({ started: workers.started, terminated: workers.terminated }).toStrictEqual({ started: 2, terminated: 1 });
    client.dispose();
  });

  test("replaces a worker that fails to start", async () => {
    const workers = new FakeWorkers([undefined, answering(ANALYSIS)]);
    const client = new AnalysisClient(workers.start);
    const pending = client.analyze(REQUEST);

    workers.failToStart(0);

    await expect(pending).rejects.toThrow("The analysis worker could not start.");
    await expect(client.analyze(REQUEST)).resolves.toStrictEqual(ANALYSIS);
    client.dispose();
  });
});

class FakeWorkers {
  readonly #apis: readonly (AnalysisApi | undefined)[];
  readonly #ports: MessagePort[] = [];
  #terminated = 0;

  constructor(apis: readonly (AnalysisApi | undefined)[]) {
    this.#apis = apis;
  }

  get started(): number {
    return this.#ports.length;
  }

  get terminated(): number {
    return this.#terminated;
  }

  readonly start = (): AnalysisWorker => {
    const { port1, port2 } = new MessageChannel();
    const api = this.#apis[this.#ports.length];

    if (api !== undefined) {
      expose(api, port2);
    }

    this.#ports.push(port1);

    return {
      postMessage: port1.postMessage.bind(port1),
      addEventListener: port1.addEventListener.bind(port1),
      removeEventListener: port1.removeEventListener.bind(port1),
      start: port1.start.bind(port1),
      terminate: () => {
        this.#terminated += 1;
        port1.close();
        port2.close();
      },
    };
  };

  failToStart(index: number): void {
    this.#ports[index]?.dispatchEvent(new Event("error"));
  }
}

function answering(analysis: Analysis): AnalysisApi {
  return {
    analyze: () => Promise.resolve(analysis),
    defaultSettings: () => Promise.resolve({}),
  };
}

function failing(rejection: Error): AnalysisApi {
  return {
    analyze: () => Promise.reject(rejection),
    defaultSettings: () => Promise.reject(rejection),
  };
}
