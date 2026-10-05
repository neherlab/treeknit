import type {
  AnalysisRequest,
  NumberSetting,
  Settings,
  SettingsSchema,
  Summary,
  ToggleSetting,
} from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import {
  createWorkspaceStore,
  type RestoredWorkspace,
  selectRequest,
  selectStale,
  type WorkspaceServices,
  type WorkspaceStore,
} from "../store";

const DEFAULTS: Settings = {
  gamma: 2,
  seqLengths: null,
  nMcmcIt: 25,
  resolve: "strict",
  preResolve: false,
  rounds: 1,
  finalRound: true,
  likelihood: true,
  naive: false,
  seed: 1,
};

const SEQ_LENGTH_DEFAULT = 1000;

const HA = "((A,B),(C,(D,X)));";

const NA = "((A,(B,X)),(C,D));";

const SUMMARY: Summary = { pairs: [], arg: { status: "built", reassortments: 1 }, diagnostics: [] };

describe("workspace store", () => {
  test("labels added trees through the label service and keeps their sources", async () => {
    const store = newStore();

    await store.getState().addTrees([
      { newick: HA, source: { kind: "file", name: "ha.nwk" } },
      { newick: NA, source: { kind: "example", name: "na.nwk" } },
      { newick: HA, source: { kind: "paste" } },
    ]);

    expect(store.getState().trees).toStrictEqual([
      { id: "tree-1", label: "ha", newick: HA, source: { kind: "file", name: "ha.nwk" } },
      { id: "tree-2", label: "na", newick: NA, source: { kind: "example", name: "na.nwk" } },
      { id: "tree-3", label: "tree", newick: HA, source: { kind: "paste" } },
    ]);
  });

  test("uses the label of a pasted tree as given", async () => {
    const store = newStore();

    await store.getState().addTrees([{ newick: HA, source: { kind: "paste" }, label: "segment.4" }]);

    expect(store.getState().trees.map(({ label }) => label)).toStrictEqual(["segment.4"]);
  });

  test("serializes concurrent adds, so two adds of one file name get distinct labels", async () => {
    const store = newStore();

    await Promise.all([
      store.getState().addTrees([{ newick: HA, source: { kind: "file", name: "ha.nwk" } }]),
      store.getState().addTrees([{ newick: NA, source: { kind: "file", name: "ha.nwk" } }]),
    ]);

    expect(store.getState().trees.map(({ label, newick }) => [label, newick])).toStrictEqual([
      ["ha", HA],
      ["ha_2", NA],
    ]);
  });

  test("continues the add queue after a failed add", async () => {
    const services = fakeServices();
    const store = createWorkspaceStore(services, { defaults: DEFAULTS, restored: null });

    services.failNextLabels = true;
    const failed = store.getState().addTrees([{ newick: HA, source: { kind: "file", name: "ha.nwk" } }]);
    const next = store.getState().addTrees([{ newick: NA, source: { kind: "file", name: "na.nwk" } }]);

    await expect(failed).rejects.toThrow("labels failed");
    await next;
    expect(store.getState().trees.map(({ label }) => label)).toStrictEqual(["na"]);
  });

  test("renames a tree and replaces its text and source", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);

    store.getState().renameTree("tree-1", "HA segment");
    store.getState().replaceTree("tree-2", "(A,B,C);", { kind: "file", name: "na_new.nwk" });

    expect(store.getState().trees.map(({ label, newick, source }) => ({ label, newick, source }))).toStrictEqual([
      { label: "HA segment", newick: HA, source: { kind: "file", name: "ha.nwk" } },
      { label: "na", newick: "(A,B,C);", source: { kind: "file", name: "na_new.nwk" } },
    ]);
  });

  test("reorders trees together with their sequence lengths", async () => {
    const store = await storeWith(["a.nwk", "b.nwk", "c.nwk"]);

    await store.getState().setSeqLengthsEnabled(true);
    store.getState().setSettings({ ...store.getState().settings, seqLengths: [10, 20, 30] });
    store.getState().reorderTrees(["tree-3", "tree-1", "tree-2"]);

    expect({
      labels: store.getState().trees.map(({ label }) => label),
      seqLengths: store.getState().settings.seqLengths,
    }).toStrictEqual({ labels: ["c", "a", "b"], seqLengths: [30, 10, 20] });
  });

  test("ignores an order that is not a permutation of the trees", async () => {
    const store = await storeWith(["a.nwk", "b.nwk"]);
    const before = store.getState().trees;

    store.getState().reorderTrees(["tree-1", "tree-1"]);
    store.getState().reorderTrees(["tree-1"]);
    store.getState().reorderTrees(["tree-1", "tree-9"]);

    expect(store.getState().trees).toBe(before);
  });

  test("removes a tree with its sequence length and undo puts both back at their index", async () => {
    const store = await storeWith(["a.nwk", "b.nwk", "c.nwk"]);

    await store.getState().setSeqLengthsEnabled(true);
    store.getState().setSettings({ ...store.getState().settings, seqLengths: [10, 20, 30] });
    store.getState().removeTree("tree-2");

    const removed = {
      labels: store.getState().trees.map(({ label }) => label),
      seqLengths: store.getState().settings.seqLengths,
    };

    store.getState().restoreUndo();

    expect({
      removed,
      restored: {
        labels: store.getState().trees.map(({ label }) => label),
        seqLengths: store.getState().settings.seqLengths,
      },
      undo: store.getState().undo,
    }).toStrictEqual({
      removed: { labels: ["a", "c"], seqLengths: [10, 30] },
      restored: { labels: ["a", "b", "c"], seqLengths: [10, 20, 30] },
      undo: null,
    });
  });

  test("the next edit after a removal drops the undo entry", async () => {
    const store = await storeWith(["a.nwk", "b.nwk", "c.nwk"]);

    store.getState().removeTree("tree-2");
    store.getState().renameTree("tree-1", "first");
    store.getState().restoreUndo();

    expect(store.getState().trees.map(({ label }) => label)).toStrictEqual(["first", "c"]);
  });

  test("appends the schema default sequence length for an added tree", async () => {
    const store = await storeWith(["a.nwk", "b.nwk"]);

    await store.getState().setSeqLengthsEnabled(true);
    await store.getState().addTrees([{ newick: HA, source: { kind: "file", name: "c.nwk" } }]);
    const enabled = store.getState().settings.seqLengths;

    await store.getState().setSeqLengthsEnabled(false);

    expect({ enabled, disabled: store.getState().settings.seqLengths }).toStrictEqual({
      enabled: [SEQ_LENGTH_DEFAULT, SEQ_LENGTH_DEFAULT, SEQ_LENGTH_DEFAULT],
      disabled: null,
    });
  });

  test("clear removes the trees, resets the settings, and undo restores the workspace with its result", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);

    store.getState().setSettings({ ...DEFAULTS, gamma: 3 });
    finishRun(store, 1);
    const before = store.getState();

    store.getState().clear();

    const cleared = {
      trees: store.getState().trees,
      settings: store.getState().settings,
      result: store.getState().result,
    };

    store.getState().restoreUndo();

    expect({
      cleared,
      restored: { trees: store.getState().trees, settings: store.getState().settings, result: store.getState().result },
    }).toStrictEqual({
      cleared: { trees: [], settings: DEFAULTS, result: null },
      restored: { trees: before.trees, settings: before.settings, result: before.result },
    });
  });

  test("loading a request replaces the workspace with session trees, and undo brings the old one back", async () => {
    const store = await storeWith(["ha.nwk"]);

    const request: AnalysisRequest = {
      trees: [
        { label: "x", newick: HA },
        { label: "y", newick: NA },
      ],
      settings: { gamma: 5 },
    };

    store.getState().loadRequest(request);
    const loaded = store.getState();

    store.getState().restoreUndo();

    expect({
      loaded: loaded.trees.map(({ label, source }) => ({ label, source })),
      gamma: loaded.settings.gamma,
      resolve: loaded.settings.resolve,
      restored: store.getState().trees.map(({ label }) => label),
    }).toStrictEqual({
      loaded: [
        { label: "x", source: { kind: "session" } },
        { label: "y", source: { kind: "session" } },
      ],
      gamma: 5,
      resolve: "strict",
      restored: ["ha"],
    });
  });

  test("a result is stale exactly when the current request differs from the one that ran", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);

    finishRun(store, 1);
    const fresh = selectStale(store.getState());

    store.getState().renameTree("tree-1", "other");
    const renamed = selectStale(store.getState());

    store.getState().renameTree("tree-1", "ha");
    const renamedBack = selectStale(store.getState());

    store.getState().setSettings({ ...store.getState().settings, seed: 2 });

    expect({ fresh, renamed, renamedBack, reseeded: selectStale(store.getState()) }).toStrictEqual({
      fresh: false,
      renamed: true,
      renamedBack: false,
      reseeded: true,
    });
  });

  test("keeps the request that ran, with complete settings", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);
    const request = selectRequest(store.getState());

    finishRun(store, 1);

    expect(store.getState().result?.request).toStrictEqual({
      trees: [
        { label: "ha", newick: HA },
        { label: "na", newick: HA },
      ],
      settings: DEFAULTS,
    });
    expect(store.getState().result?.request).toStrictEqual(request);
  });

  test("records progress and the outcome of the current run only", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);
    const progress = { phase: "pairs" as const, fraction: 0.5, round: 1, rounds: 1, pair: 1, pairs: 1 };

    store.getState().runStarted(2, selectRequest(store.getState()), 1000);
    store.getState().runProgressed(1, { ...progress, fraction: 0.9 });
    store.getState().runProgressed(2, progress);
    store.getState().runFinished(1, { status: "succeeded", sessionId: 1, summary: SUMMARY });
    const running = store.getState().run;

    store.getState().runFinished(2, { status: "failed", kind: "cancelled", message: "Run cancelled." });

    expect({ running, finished: store.getState().run, result: store.getState().result }).toStrictEqual({
      running: { status: "running", runId: 2, request: selectRequest(store.getState()), progress, startedAt: 1000 },
      finished: { status: "failed", kind: "cancelled", message: "Run cancelled." },
      result: null,
    });
  });

  test("a failed run keeps the earlier result", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);

    finishRun(store, 1);
    store.getState().runStarted(2, selectRequest(store.getState()), 0);
    store.getState().runFinished(2, { status: "failed", kind: "internal", message: "boom" });

    expect(store.getState().result?.sessionId).toBe(1);
  });

  test("a lost session clears its result with the lost-results message", async () => {
    const store = await storeWith(["ha.nwk", "na.nwk"]);

    finishRun(store, 1);
    store.getState().resultLost(7);
    const kept = store.getState().result?.sessionId;

    store.getState().resultLost(1);

    expect({ kept, result: store.getState().result, run: store.getState().run }).toStrictEqual({
      kept: 1,
      result: null,
      run: {
        status: "failed",
        kind: "internal",
        message: "The results were lost because of an internal error. Run again.",
      },
    });
  });

  test("starts from a restored workspace with its sources and the restored status", async () => {
    const restored: RestoredWorkspace = {
      request: {
        trees: [
          { label: "ha", newick: HA },
          { label: "na", newick: NA },
        ],
        settings: { ...DEFAULTS, seqLengths: [5, 6] },
      },
      sources: [{ kind: "file", name: "ha.nwk" }],
    };

    const store = createWorkspaceStore(fakeServices(), { defaults: DEFAULTS, restored });
    const start = store.getState();

    await store.getState().addTrees([{ newick: HA, source: { kind: "paste" } }]);

    expect({
      trees: start.trees,
      seqLengths: start.settings.seqLengths,
      restored: start.restored,
      afterEdit: store.getState().restored,
      newId: store.getState().trees.at(-1)?.id,
    }).toStrictEqual({
      trees: [
        { id: "tree-1", label: "ha", newick: HA, source: { kind: "file", name: "ha.nwk" } },
        { id: "tree-2", label: "na", newick: NA, source: { kind: "session" } },
      ],
      seqLengths: [5, 6],
      restored: true,
      afterEdit: false,
      newId: "tree-3",
    });
  });
});

class FakeServices implements WorkspaceServices {
  failNextLabels = false;

  async treeLabels(fileNames: string[], existingLabels: string[]): Promise<string[]> {
    await Promise.resolve();

    if (this.failNextLabels) {
      this.failNextLabels = false;

      throw new Error("labels failed");
    }

    const taken = new Set(existingLabels);

    return fileNames.map((name) => {
      const stem = name.replace(/\.[^.]*$/u, "");
      let label = stem;

      for (let suffix = 2; taken.has(label); suffix += 1) {
        label = `${stem}_${String(suffix)}`;
      }

      taken.add(label);

      return label;
    });
  }

  async settingsSchema(): Promise<SettingsSchema> {
    await Promise.resolve();

    return schemaWith(SEQ_LENGTH_DEFAULT);
  }
}

async function storeWith(fileNames: string[]): Promise<WorkspaceStore> {
  const store = newStore();

  await store.getState().addTrees(fileNames.map((name) => ({ newick: HA, source: { kind: "file", name } })));

  return store;
}

function newStore(): WorkspaceStore {
  return createWorkspaceStore(fakeServices(), { defaults: DEFAULTS, restored: null });
}

function finishRun(store: WorkspaceStore, runId: number): void {
  store.getState().runStarted(runId, selectRequest(store.getState()), 0);
  store.getState().runFinished(runId, { status: "succeeded", sessionId: runId, summary: SUMMARY });
}

function fakeServices(): FakeServices {
  return new FakeServices();
}

function schemaWith(seqLengthDefault: number): SettingsSchema {
  const number: NumberSetting = {
    default: 0,
    min: 0,
    minExclusive: false,
    max: null,
    step: 1,
    applies: true,
    reason: null,
    help: "",
  };
  const toggle: ToggleSetting = { default: false, applies: true, reason: null, help: "" };

  return {
    settings: {
      gamma: number,
      seqLengths: { ...number, default: seqLengthDefault },
      nMcmcIt: number,
      rounds: number,
      seed: number,
      preResolve: toggle,
      finalRound: toggle,
      likelihood: toggle,
      naive: toggle,
    },
    modes: [],
    treeOrderHelp: "",
  };
}
