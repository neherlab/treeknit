import { describe, expect, test } from "vitest";

import {
  formatNodeRef,
  isViewAvailable,
  NO_WORKSPACE,
  type NodeRef,
  parseSearch,
  resolvePair,
  resolveWorkspaceSearch,
  selectPair,
  stringifySearch,
  type WorkspaceAvailability,
  type WorkspaceSearch,
  workspaceSearchSchema,
  viewFitsTreeCount,
  type WorkspaceView,
} from "../search";

const DEFAULT_SEARCH: WorkspaceSearch = {
  view: "overview",
  pair: 0,
  version: "resolved",
  x: "div",
  labels: "auto",
};

function parseUrl(query: string): WorkspaceSearch {
  return workspaceSearchSchema.parse(parseSearch(query));
}

function availability(overrides: Partial<WorkspaceAvailability>): WorkspaceAvailability {
  return { ...NO_WORKSPACE, ...overrides };
}

const TWO_TREE_RESULT = availability({ hasResult: true, treeCount: 2, resultTreeCount: 2, pairCount: 1 });

const THREE_TREE_RESULT = availability({ hasResult: true, treeCount: 3, resultTreeCount: 3, pairCount: 3 });

describe("workspaceSearchSchema", () => {
  test("gives the defaults for an empty query", () => {
    expect(parseUrl("")).toStrictEqual(DEFAULT_SEARCH);
  });

  test("reads every parameter from the query", () => {
    const expected: WorkspaceSearch = {
      view: "tanglegram",
      pair: 2,
      version: "imputed",
      x: "depth",
      labels: "off",
      mcc: 4,
      leaf: "A/New York/392/2004",
      node: { side: "left", name: "NODE_3" },
    };

    expect(
      parseUrl(
        "?view=tanglegram&pair=2&version=imputed&x=depth&labels=off&mcc=4&leaf=A%2FNew%20York%2F392%2F2004&node=left%3ANODE_3",
      ),
    ).toStrictEqual(expected);
  });

  test.each([
    { name: "an unknown view", query: "view=map", key: "view", expected: "overview" },
    { name: "a negative pair", query: "pair=-1", key: "pair", expected: 0 },
    { name: "a fractional pair", query: "pair=1.5", key: "pair", expected: 0 },
    { name: "a pair that is not a number", query: "pair=first", key: "pair", expected: 0 },
    { name: "an empty pair", query: "pair=", key: "pair", expected: 0 },
    { name: "an unknown tree version", query: "version=final", key: "version", expected: "resolved" },
    { name: "an unknown x scale", query: "x=time", key: "x", expected: "div" },
    { name: "an unknown label mode", query: "labels=some", key: "labels", expected: "auto" },
  ] as const)("falls back to the default for $name", ({ query, key, expected }) => {
    expect(parseUrl(`?${query}`)[key]).toStrictEqual(expected);
  });

  test.each([
    { name: "an mcc that is not an integer", query: "mcc=two", key: "mcc" },
    { name: "an empty mcc", query: "mcc=", key: "mcc" },
    { name: "a negative mcc", query: "mcc=-1", key: "mcc" },
    { name: "an empty leaf", query: "leaf=", key: "leaf" },
    { name: "a node without a side", query: "node=NODE_3", key: "node" },
    { name: "a node with an unknown side", query: "node=top%3ANODE_3", key: "node" },
    { name: "a node without a name", query: "node=arg%3A", key: "node" },
  ] as const)("drops $name", ({ query, key }) => {
    expect(parseUrl(`?${query}`)[key]).toBeUndefined();
  });

  test("keeps a leaf name made of digits as text", () => {
    expect(parseUrl("?leaf=123").leaf).toBe("123");
  });

  test.each(["[1]", "true", "1e3", "007"])("keeps the leaf name %s as text", (leaf) => {
    expect(parseUrl(`?leaf=${encodeURIComponent(leaf)}`).leaf).toBe(leaf);
  });

  test("reads the last value of a repeated parameter", () => {
    expect(parseUrl("?view=arg&view=files").view).toBe("files");
  });

  test("accepts numbers set in memory by a navigation", () => {
    expect(workspaceSearchSchema.parse({ pair: 3, mcc: 0 })).toStrictEqual({ ...DEFAULT_SEARCH, pair: 3, mcc: 0 });
  });
});

describe("stringifySearch", () => {
  test("writes plain values that read back unchanged", () => {
    const search: WorkspaceSearch = {
      ...DEFAULT_SEARCH,
      view: "arg",
      pair: 1,
      leaf: "007",
      node: { side: "arg", name: "hybrid,1" },
    };

    expect(parseUrl(stringifySearch(search))).toStrictEqual(search);
  });

  test("writes the leaf name without JSON quotes", () => {
    expect(stringifySearch({ leaf: "ha" })).toBe("?leaf=ha");
  });
});

describe("a change to the written search", () => {
  test("keeps the values that the availability hides", () => {
    const written = parseUrl("?pair=2&leaf=X&mcc=1");
    const changed = parseUrl(stringifySearch({ ...written, view: "files" }));

    expect(resolveWorkspaceSearch(written, NO_WORKSPACE)).toStrictEqual(DEFAULT_SEARCH);
    expect(changed).toStrictEqual({ ...DEFAULT_SEARCH, view: "files", pair: 2, leaf: "X", mcc: 1 });
  });
});

describe("the node parameter", () => {
  test.each([
    { text: "left:NODE_3", expected: { side: "left", name: "NODE_3" } },
    { text: "right:a:b", expected: { side: "right", name: "a:b" } },
    { text: "arg:hybrid 1", expected: { side: "arg", name: "hybrid 1" } },
  ] as const)("reads $text", ({ text, expected }) => {
    expect(parseUrl(`?node=${encodeURIComponent(text)}`).node).toStrictEqual(expected);
  });

  test("reads back what formatNodeRef writes", () => {
    const node: NodeRef = { side: "right", name: "NODE_12:x" };

    expect(parseUrl(`?node=${encodeURIComponent(formatNodeRef(node))}`).node).toStrictEqual(node);
  });

  test("keeps a node set in memory by a navigation", () => {
    const node: NodeRef = { side: "left", name: "NODE_3" };

    expect(workspaceSearchSchema.parse({ node }).node).toStrictEqual(node);
  });

  test("writes the node as side and name", () => {
    expect(parseSearch(stringifySearch({ node: { side: "left", name: "NODE 3" } }))).toStrictEqual({
      node: "left:NODE 3",
    });
  });
});

describe("viewFitsTreeCount", () => {
  test.each([
    { view: "arg", treeCount: 2, expected: true },
    { view: "arg", treeCount: 3, expected: false },
    { view: "constellation", treeCount: 2, expected: false },
    { view: "constellation", treeCount: 3, expected: true },
    { view: "tanglegram", treeCount: 1, expected: true },
  ] as const)("$view for $treeCount trees: $expected", ({ view, treeCount, expected }) => {
    expect(viewFitsTreeCount(view, treeCount)).toBe(expected);
  });
});

describe("isViewAvailable", () => {
  test.each([
    { view: "overview", workspace: "no result", available: NO_WORKSPACE, expected: true },
    { view: "tanglegram", workspace: "no result", available: NO_WORKSPACE, expected: false },
    { view: "mccs", workspace: "no result", available: NO_WORKSPACE, expected: false },
    { view: "files", workspace: "no result", available: NO_WORKSPACE, expected: false },
    { view: "diagnostics", workspace: "no result", available: NO_WORKSPACE, expected: false },
    { view: "arg", workspace: "no result", available: NO_WORKSPACE, expected: false },
    { view: "tanglegram", workspace: "a two-tree result", available: TWO_TREE_RESULT, expected: true },
    { view: "arg", workspace: "a two-tree result", available: TWO_TREE_RESULT, expected: true },
    { view: "constellation", workspace: "a two-tree result", available: TWO_TREE_RESULT, expected: false },
    { view: "files", workspace: "a two-tree result", available: TWO_TREE_RESULT, expected: true },
    { view: "arg", workspace: "a three-tree result", available: THREE_TREE_RESULT, expected: false },
    { view: "constellation", workspace: "a three-tree result", available: THREE_TREE_RESULT, expected: true },
  ] as const)("$view with $workspace: $expected", ({ view, available, expected }) => {
    expect(isViewAvailable(view, available)).toBe(expected);
  });
});

describe("resolveWorkspaceSearch", () => {
  test.each([
    { view: "arg", workspace: "a three-tree result", available: THREE_TREE_RESULT },
    { view: "constellation", workspace: "a two-tree result", available: TWO_TREE_RESULT },
    { view: "tanglegram", workspace: "no result", available: NO_WORKSPACE },
    { view: "diagnostics", workspace: "no result", available: NO_WORKSPACE },
  ] as const)("shows the overview for $view with $workspace", ({ view, available }) => {
    expect(resolveWorkspaceSearch({ ...DEFAULT_SEARCH, view }, available).view).toBe("overview");
  });

  test("keeps an available view", () => {
    const view: WorkspaceView = "constellation";

    expect(resolveWorkspaceSearch({ ...DEFAULT_SEARCH, view }, THREE_TREE_RESULT).view).toBe(view);
  });

  test.each([
    { pair: 3, expected: 0 },
    { pair: 2, expected: 2 },
  ])("turns pair $pair of three pairs into $expected", ({ pair, expected }) => {
    expect(resolveWorkspaceSearch({ ...DEFAULT_SEARCH, pair }, THREE_TREE_RESULT).pair).toBe(expected);
  });

  test("clears an mcc, a leaf, and a node absent from the pair", () => {
    const search: WorkspaceSearch = { ...DEFAULT_SEARCH, mcc: 7, leaf: "X", node: { side: "left", name: "NODE_1" } };

    expect(resolveWorkspaceSearch(search, TWO_TREE_RESULT)).toStrictEqual(DEFAULT_SEARCH);
  });

  test("keeps an mcc, a leaf, and a node present in the pair", () => {
    const search: WorkspaceSearch = {
      ...DEFAULT_SEARCH,
      view: "tanglegram",
      mcc: 1,
      leaf: "X",
      node: { side: "left", name: "NODE_1" },
    };

    const present = availability({
      ...TWO_TREE_RESULT,
      mccExists: (pair, mcc) => pair === 0 && mcc === 1,
      leafExists: (pair, leaf) => pair === 0 && leaf === "X",
      nodeExists: (pair, node) => pair === 0 && node.side === "left" && node.name === "NODE_1",
    });

    expect(resolveWorkspaceSearch(search, present)).toStrictEqual(search);
  });

  test("checks the mcc, leaf, and node against the pair that the fallback chose", () => {
    const checkedPairs: number[] = [];

    const recording = availability({
      ...TWO_TREE_RESULT,
      mccExists: (pair) => checkedPairs.push(pair) > 0,
      leafExists: (pair) => checkedPairs.push(pair) > 0,
      nodeExists: (pair) => checkedPairs.push(pair) > 0,
    });

    resolveWorkspaceSearch(
      { ...DEFAULT_SEARCH, pair: 5, mcc: 1, leaf: "X", node: { side: "arg", name: "H" } },
      recording,
    );

    expect(checkedPairs).toStrictEqual([0, 0, 0]);
  });
});

describe("selectPair", () => {
  test("sets the pair and clears the mcc and the node, keeping the leaf", () => {
    const search: WorkspaceSearch = {
      ...DEFAULT_SEARCH,
      view: "tanglegram",
      mcc: 2,
      leaf: "X",
      node: { side: "right", name: "NODE_4" },
    };

    expect(selectPair(search, 1)).toStrictEqual({ ...DEFAULT_SEARCH, view: "tanglegram", pair: 1, leaf: "X" });
  });

  test("drops the kept leaf when the new pair lacks it", () => {
    const leafOnlyInPairZero = availability({ ...THREE_TREE_RESULT, leafExists: (pair) => pair === 0 });

    const resolved = resolveWorkspaceSearch(selectPair({ ...DEFAULT_SEARCH, leaf: "X" }, 2), leafOnlyInPairZero);

    expect(resolved).toStrictEqual({ ...DEFAULT_SEARCH, pair: 2 });
  });
});

describe("resolvePair", () => {
  test("keeps a pair in range and falls back to the first pair otherwise", () => {
    expect([resolvePair(2, 3), resolvePair(3, 3), resolvePair(0, 0), resolvePair(5, 0)]).toStrictEqual([2, 0, 0, 0]);
  });
});
