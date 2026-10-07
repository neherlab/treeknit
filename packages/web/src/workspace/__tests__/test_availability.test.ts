import type { ArgView, PairView, Summary } from "@neherlab/treeknit-wasm";
import { defaultScheduler, notifyManager, QueryClient } from "@tanstack/react-query";
import { afterEach, describe, expect, test } from "vitest";

import { type LoadedViews, subscribeToQueryCache, workspaceAvailability } from "../availability";
import { resolveWorkspaceSearch, type WorkspaceSearch, type WrittenSearch } from "../search";
import type { RunResult } from "../store";

const SUMMARY: Summary = {
  pairs: [
    {
      index: 0,
      trees: [0, 1],
      labels: ["ha", "na"],
      mccCount: 2,
      mccs: [["X"], ["A", "B", "C", "D"]],
      imputedCount: 0,
      ambiguousCount: 0,
    },
  ],
  arg: { status: "built", reassortments: 1 },
  noReassortment: false,
  diagnostics: [],
};

const RESULT: RunResult = {
  sessionId: 1,
  summary: SUMMARY,
  request: {
    trees: [
      { label: "ha", newick: "((A,B),(C,(D,X)));" },
      { label: "na", newick: "((A,(B,X)),(C,D));" },
    ],
  },
  durationMs: 4200,
};

const NOTHING_LOADED: LoadedViews = { pairView: () => undefined, argView: () => undefined };

const SEARCH: WrittenSearch = {
  view: "tanglegram",
  version: "resolved",
  scale: "div",
  labels: "auto",
  mcc: 1,
  leaf: "X",
  node: { side: "left", name: "NODE_1" },
};

const RESOLVED: WorkspaceSearch = { ...SEARCH, pair: 0, show: "both" };

describe("workspace availability", () => {
  test("without a result has the tree count and nothing else", () => {
    const availability = workspaceAvailability(3, null, NOTHING_LOADED);

    expect({
      counts: [availability.hasResult, availability.treeCount, availability.resultTreeCount, availability.pairLabels],
      mcc: availability.mccExists(0, 0),
      leaf: availability.leafExists(0, "X"),
      node: availability.nodeExists(0, { side: "left", name: "NODE_1" }),
    }).toStrictEqual({ counts: [false, 3, 0, []], mcc: false, leaf: false, node: false });
  });

  test("takes the tree and pair counts of the result, not of the current trees", () => {
    const availability = workspaceAvailability(5, RESULT, NOTHING_LOADED);

    expect([
      availability.hasResult,
      availability.treeCount,
      availability.resultTreeCount,
      availability.pairLabels,
    ]).toStrictEqual([true, 5, 2, [["ha", "na"]]]);
  });

  test("knows the MCC numbers and leaves of each pair", () => {
    const availability = workspaceAvailability(2, RESULT, NOTHING_LOADED);

    expect({
      mccs: [
        availability.mccExists(0, 0),
        availability.mccExists(0, 1),
        availability.mccExists(0, 2),
        availability.mccExists(1, 0),
      ],
      leaves: [
        availability.leafExists(0, "X"),
        availability.leafExists(0, "D"),
        availability.leafExists(0, "Y"),
        availability.leafExists(1, "X"),
      ],
    }).toStrictEqual({ mccs: [true, true, false, false], leaves: [true, true, false, false] });
  });

  test("keeps a node until the view of its pair is loaded, then checks its name among the internal nodes of its side", () => {
    const loaded: LoadedViews = {
      pairView: () => pairViewWith(["NODE_1", "A"], ["NODE_2", "A"]),
      argView: () => undefined,
    };

    const unknown = workspaceAvailability(2, RESULT, NOTHING_LOADED);
    const known = workspaceAvailability(2, RESULT, loaded);

    expect({
      unknown: unknown.nodeExists(0, { side: "left", name: "NODE_9" }),
      left: known.nodeExists(0, { side: "left", name: "NODE_1" }),
      wrongSide: known.nodeExists(0, { side: "right", name: "NODE_1" }),
      right: known.nodeExists(0, { side: "right", name: "NODE_2" }),
      outOfRange: known.nodeExists(1, { side: "left", name: "NODE_1" }),
      leaf: known.nodeExists(0, { side: "left", name: "A" }),
    }).toStrictEqual({ unknown: true, left: true, wrongSide: false, right: true, outOfRange: false, leaf: false });
  });

  test("checks ARG nodes against the loaded ARG view, and none exist without an ARG", () => {
    const withArg = workspaceAvailability(2, RESULT, {
      pairView: () => undefined,
      argView: () => argViewWith(["GlobalRoot", "A"]),
    });

    const withoutArg = workspaceAvailability(2, RESULT, { pairView: () => undefined, argView: () => null });

    expect([
      withArg.nodeExists(0, { side: "arg", name: "GlobalRoot" }),
      withArg.nodeExists(0, { side: "arg", name: "NODE_1" }),
      withoutArg.nodeExists(0, { side: "arg", name: "GlobalRoot" }),
    ]).toStrictEqual([true, false, false]);
  });

  test("resolves the URL against the result: present values stay, missing ones are cleared", () => {
    const loaded: LoadedViews = { pairView: () => pairViewWith(["A"], ["A"]), argView: () => undefined };

    expect({
      kept: resolveWorkspaceSearch(SEARCH, workspaceAvailability(2, RESULT, NOTHING_LOADED)),
      cleared: resolveWorkspaceSearch({ ...SEARCH, mcc: 5, leaf: "Y" }, workspaceAvailability(2, RESULT, loaded)),
      noResult: resolveWorkspaceSearch(SEARCH, workspaceAvailability(2, null, NOTHING_LOADED)),
    }).toStrictEqual({
      kept: RESOLVED,
      cleared: { view: "tanglegram", pair: 0, version: "resolved", scale: "div", labels: "auto", show: "both" },
      noResult: { view: "overview", pair: 0, version: "resolved", scale: "div", labels: "auto", show: "both" },
    });
  });
});

describe("query cache subscription", () => {
  afterEach(() => {
    notifyManager.setScheduler(defaultScheduler);
  });

  test("defers the change notice of a query that a render creates", () => {
    const scheduled: (() => void)[] = [];

    notifyManager.setScheduler((callback) => {
      scheduled.push(callback);
    });

    const queryClient = new QueryClient();
    const notices: string[] = [];

    const unsubscribe = subscribeToQueryCache(queryClient, () => {
      notices.push("changed");
    });

    queryClient.getQueryCache().build(queryClient, { queryKey: ["inspectTree", "(A,B);"] });
    const duringRender = [...notices];

    for (const callback of scheduled) {
      callback();
    }

    unsubscribe();

    expect({ duringRender, afterRender: notices }).toStrictEqual({ duringRender: [], afterRender: ["changed"] });
  });
});

function pairViewWith(left: string[], right: string[]): PairView {
  return {
    left: { label: "ha", nodes: drawNodes(left) },
    right: { label: "na", nodes: drawNodes(right) },
    links: [],
    blocks: [],
    mccs: [],
    scale: "div",
    legend: [],
    // oxlint-disable-next-line anti-slop/no-shape-in-symbol-names -- field name of the generated PairView type
    shapes: {
      left: { elbows: [], marks: [], leaders: [] },
      right: { elbows: [], marks: [], leaders: [] },
      links: [],
      ribbons: [],
    },
  };
}

function drawNodes([root, ...leaves]: string[]): PairView["left"]["nodes"] {
  const children = leaves.map((_, leaf) => leaf + 1);

  return [
    root === undefined ? [] : [drawNode(root, null, children, Math.max(0, leaves.length - 1) / 2)],
    leaves.map((name, leaf) => drawNode(name, 0, [], leaf)),
  ].flat();
}

function drawNode(
  name: string,
  parent: number | null,
  children: number[],
  y: number,
): PairView["left"]["nodes"][number] {
  return {
    name,
    shortName: name,
    parent,
    children,
    branchLength: null,
    meanLength: null,
    xDiv: 0,
    xDepth: 0,
    y,
    leaf: children.length === 0,
    cladeSize: Math.max(1, children.length),
    added: false,
    imputed: false,
    mcc: null,
    mccBreak: false,
  };
}

function argViewWith(labels: string[]): ArgView {
  return {
    nodes: labels.map((label) => ({
      label,
      shortLabel: label,
      parents: [null, null],
      children: [],
      tau: [null, null],
      hybrid: false,
      leaf: false,
      segments: [0, 1],
      xDiv: 0,
      xDepth: 0,
      y: 0,
    })),
    edges: [],
    root: 0,
    rootCase: "synthetic",
    scale: "div",
    legend: [],
    // oxlint-disable-next-line anti-slop/no-shape-in-symbol-names -- field name of the generated ArgView type
    shapes: { edges: [], marks: [], leaders: [] },
  };
}
