import type { Summary } from "@neherlab/treeknit-wasm";
import * as fc from "fast-check";
import { describe, expect, test } from "vitest";

import type { RunOutcome } from "../../analysis/client";
import {
  formatNodeRef,
  isViewAvailable,
  NO_WORKSPACE,
  type NodeRef,
  type PairRef,
  parseSearch,
  resolveWorkspaceSearch,
  searchAfterRun,
  selectPair,
  stringifySearch,
  type WorkspaceAvailability,
  type WorkspaceSearch,
  workspaceSearchSchema,
  viewFitsTreeCount,
  type WorkspaceView,
  withSelectionPair,
  type WrittenSearch,
} from "../search";
import { readQuery, writeQuery } from "../searchQuery";

const DEFAULT_SEARCH: WrittenSearch = {
  view: "overview",
  version: "resolved",
  scale: "div",
  labels: "auto",
};

const DEFAULT_RESOLVED: WorkspaceSearch = { ...DEFAULT_SEARCH, pair: 0, show: "both" };

const TWO_TREE_PAIRS: readonly PairRef[] = [["ha", "na"]];

const THREE_TREE_PAIRS: readonly PairRef[] = [
  ["ha", "na"],
  ["ha", "pb1"],
  ["na", "pb1"],
];

const TWO_TREE_RESULT = availability({ hasResult: true, treeCount: 2, resultTreeCount: 2, pairLabels: TWO_TREE_PAIRS });

const THREE_TREE_RESULT = availability({
  hasResult: true,
  treeCount: 3,
  resultTreeCount: 3,
  pairLabels: THREE_TREE_PAIRS,
});

describe("readQuery and writeQuery", () => {
  test("keep repeated keys in order and keys without a value as flags", () => {
    const query = "tree=ha=https://x/a.nwk&tree=https://x/b.nwk&run&gamma=3";

    expect({ read: readQuery(query), written: writeQuery(readQuery(query)) }).toStrictEqual({
      read: [
        { key: "tree", value: "ha=https://x/a.nwk" },
        { key: "tree", value: "https://x/b.nwk" },
        { key: "run", value: true },
        { key: "gamma", value: "3" },
      ],
      written: query,
    });
  });

  test("read a plus as a space and an encoded plus as a plus, as URLSearchParams does", () => {
    expect(readQuery("a=x+y&b=%2B")).toStrictEqual([
      { key: "a", value: "x y" },
      { key: "b", value: "+" },
    ]);
  });

  test.each([
    ["a leaf name with spaces and slashes", "leaf", "A/New York/392/2004", "leaf=A/New%20York/392/2004"],
    ["an Auspice value, a query of its own", "auspice", "legend=open&l=radial", "auspice=legend=open%26l=radial"],
    ["the characters that a reader misreads", "s", "a+b#c%d", "s=a%2Bb%23c%25d"],
    ["a non-ASCII name", "leaf", "Ä", "leaf=%C3%84"],
    ["a Newick text", "tree", "data:,((A,B),(C,D));", "tree=data:,((A,B),(C,D));"],
  ])("write %s readably", (_case, key, value, expected) => {
    expect(writeQuery([{ key, value }])).toBe(expected);
  });

  test("escape = in keys only", () => {
    expect(writeQuery([{ key: "a=b", value: "c=d" }])).toBe("a%3Db=c=d");
  });

  test("give back arbitrary keys and values", () => {
    const text = fc.string({ unit: "grapheme", minLength: 1 });

    fc.assert(
      fc.property(
        fc.array(
          fc.record({ key: text, value: fc.oneof(text, fc.constant(true as const)) }, { noNullPrototype: true }),
          {
            minLength: 1,
          },
        ),
        (entries) => {
          expect(readQuery(writeQuery(entries))).toStrictEqual(entries);
        },
      ),
    );
  });
});

describe("parseSearch and workspaceSearchSchema", () => {
  test("give the defaults for an empty query", () => {
    expect(parseUrl("")).toStrictEqual(DEFAULT_SEARCH);
  });

  test("read every view key from the query", () => {
    const expected: WrittenSearch = {
      view: "tanglegram",
      pair: ["ha", "pb1"],
      version: "imputed",
      scale: "depth",
      labels: "off",
      show: "pb1",
      mcc: 3,
      leaf: "A/New York/392/2004",
      node: { side: "left", name: "NODE_3" },
    };

    expect(
      parseUrl(
        "?view=tanglegram&pair=ha:pb1&version=imputed&scale=depth&labels=off&show=pb1&mcc=4&leaf=A/New%20York/392/2004&node=left:NODE_3",
      ),
    ).toStrictEqual(expected);
  });

  test("read the MCC number that the interface shows and keep its index", () => {
    expect({ read: parseUrl("?mcc=4").mcc, written: stringifySearch({ mcc: 3 }) }).toStrictEqual({
      read: 3,
      written: "?mcc=4",
    });
  });

  test.each([
    { name: "an unknown view", query: "view=map", key: "view", expected: "overview" },
    { name: "an unknown tree version", query: "version=final", key: "version", expected: "resolved" },
    { name: "an unknown scale", query: "scale=time", key: "scale", expected: "div" },
    { name: "an unknown label mode", query: "labels=some", key: "labels", expected: "auto" },
    { name: "a repeated view", query: "view=arg&view=files", key: "view", expected: "overview" },
  ] as const)("fall back to the default for $name", ({ query, key, expected }) => {
    expect(parseUrl(`?${query}`)[key]).toStrictEqual(expected);
  });

  test.each([
    { name: "an MCC that is not a number", query: "mcc=two", key: "mcc" },
    { name: "an empty MCC", query: "mcc=", key: "mcc" },
    { name: "MCC 0, which the interface does not show", query: "mcc=0", key: "mcc" },
    { name: "a negative MCC", query: "mcc=-1", key: "mcc" },
    { name: "a pair of one label", query: "pair=ha", key: "pair" },
    { name: "a pair of three labels", query: "pair=ha:na:pb1", key: "pair" },
    { name: "a pair with an empty label", query: "pair=:na", key: "pair" },
    { name: "an empty leaf", query: "leaf=", key: "leaf" },
    { name: "an empty shown tree", query: "show=", key: "show" },
    { name: "a node without a side", query: "node=NODE_3", key: "node" },
    { name: "a node with an unknown side", query: "node=top:NODE_3", key: "node" },
    { name: "a node without a name", query: "node=arg:", key: "node" },
    { name: "an empty Auspice value", query: "auspice=", key: "auspice" },
  ] as const)("drop $name", ({ query, key }) => {
    expect(parseUrl(`?${query}`)[key]).toBeUndefined();
  });

  test.each(["[1]", "true", "1e3", "007", "123"])("keep the leaf name %s as text", (leaf) => {
    expect(parseUrl(`?leaf=${encodeURIComponent(leaf)}`).leaf).toBe(leaf);
  });

  test.each([
    { text: "left:NODE_3", expected: { side: "left", name: "NODE_3" } },
    { text: "right:a:b", expected: { side: "right", name: "a:b" } },
    { text: "arg:hybrid 1", expected: { side: "arg", name: "hybrid 1" } },
  ] as const)("read the node $text", ({ text, expected }) => {
    expect(parseUrl(`?node=${encodeURIComponent(text)}`).node).toStrictEqual(expected);
  });

  test("read the Auspice value as a query of its own", () => {
    expect(parseUrl("?auspice=legend=open%26l=radial").auspice).toBe("legend=open&l=radial");
  });

  test("pass keys that the view does not own through unchanged, repeated keys and flags included", () => {
    const query = "?tree=https://x/a.nwk&tree=https://x/b.nwk&run&view=mccs&utm_source=paper";
    const parsed = parseSearch(query);

    expect({ parsed, written: stringifySearch(parsed) }).toStrictEqual({
      parsed: {
        tree: ["https://x/a.nwk", "https://x/b.nwk"],
        run: true,
        view: "mccs",
        utm_source: "paper",
      },
      written: query,
    });
  });

  test("accept typed values set in memory by a navigation", () => {
    const node: NodeRef = { side: "left", name: "NODE_3" };

    expect(workspaceSearchSchema.parse({ pair: ["ha", "na"], mcc: 0, node })).toStrictEqual({
      ...DEFAULT_SEARCH,
      pair: ["ha", "na"],
      mcc: 0,
      node,
    });
  });
});

describe("stringifySearch", () => {
  test("writes typed values that read back unchanged", () => {
    const search: WrittenSearch = {
      ...DEFAULT_SEARCH,
      view: "arg",
      pair: ["na", "pb1"],
      show: "na",
      mcc: 0,
      leaf: "007",
      node: { side: "arg", name: "hybrid,1" },
      auspice: "c=mcc&f_mcc=MCC 2",
    };

    expect(parseUrl(stringifySearch(search))).toStrictEqual(search);
  });

  test("writes the pair by its labels, the node as side and name, and leaf names without quotes", () => {
    expect(stringifySearch({ pair: ["ha", "na"], node: { side: "left", name: "NODE 3" }, leaf: "ha" })).toBe(
      "?pair=ha:na&node=left:NODE%203&leaf=ha",
    );
  });

  test("writes nothing for an empty search", () => {
    expect(stringifySearch({})).toBe("");
  });

  test("gives back arbitrary view values", () => {
    const text = fc.string({ unit: "grapheme", minLength: 1 });
    const label = text.filter((value) => !value.includes(":"));

    fc.assert(
      fc.property(
        fc.record(
          {
            pair: fc.tuple(label, label),
            show: text,
            auspice: text,
            mcc: fc.nat({ max: 1_000_000 }),
            leaf: text,
            node: fc.record({ side: fc.constantFrom("left", "right", "arg"), name: text }, { noNullPrototype: true }),
          },
          { requiredKeys: [], noNullPrototype: true },
        ),
        (values) => {
          const search = workspaceSearchSchema.parse(values);

          expect(parseUrl(stringifySearch(search))).toStrictEqual(search);
        },
      ),
    );
  });

  test("formatNodeRef writes side and name", () => {
    expect(formatNodeRef({ side: "right", name: "NODE_12:x" })).toBe("right:NODE_12:x");
  });
});

describe("a change to the written search", () => {
  test("keeps the values that the availability hides", () => {
    const written = parseUrl("?pair=ha:pb1&leaf=X&mcc=2");
    const changed = parseUrl(stringifySearch({ ...written, view: "files" }));

    expect(resolveWorkspaceSearch(written, NO_WORKSPACE)).toStrictEqual(DEFAULT_RESOLVED);
    expect(changed).toStrictEqual({ ...DEFAULT_SEARCH, view: "files", pair: ["ha", "pb1"], leaf: "X", mcc: 1 });
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
    { view: "auspice", workspace: "no result", available: NO_WORKSPACE, expected: false },
    { view: "auspice", workspace: "a two-tree result", available: TWO_TREE_RESULT, expected: true },
    { view: "auspice", workspace: "a three-tree result", available: THREE_TREE_RESULT, expected: true },
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
    { name: "the labels of a pair", pair: ["na", "pb1"], expected: 2 },
    { name: "the labels of a pair in the other order", pair: ["pb1", "ha"], expected: 1 },
    { name: "labels that name no pair", pair: ["ha", "pb2"], expected: 0 },
    { name: "the labels of the same tree twice", pair: ["ha", "ha"], expected: 0 },
  ] as const)("turns $name into pair $expected", ({ pair, expected }) => {
    expect(resolveWorkspaceSearch({ ...DEFAULT_SEARCH, pair }, THREE_TREE_RESULT).pair).toBe(expected);
  });

  test.each([
    { show: "ha", expected: "left" },
    { show: "na", expected: "right" },
    { show: "pb1", expected: "both" },
  ] as const)("shows $expected for show=$show in the pair ha, na", ({ show, expected }) => {
    expect(resolveWorkspaceSearch({ ...DEFAULT_SEARCH, pair: ["ha", "na"], show }, THREE_TREE_RESULT).show).toBe(
      expected,
    );
  });

  test("clears an mcc, a leaf, and a node absent from the pair", () => {
    const search: WrittenSearch = { ...DEFAULT_SEARCH, mcc: 7, leaf: "X", node: { side: "left", name: "NODE_1" } };

    expect(resolveWorkspaceSearch(search, TWO_TREE_RESULT)).toStrictEqual(DEFAULT_RESOLVED);
  });

  test("keeps an mcc, a leaf, and a node present in the pair", () => {
    const selection = { mcc: 1, leaf: "X", node: { side: "left", name: "NODE_1" } } as const;
    const search: WrittenSearch = { ...DEFAULT_SEARCH, view: "tanglegram", ...selection };

    const present = availability({
      ...TWO_TREE_RESULT,
      mccExists: (pair, mcc) => pair === 0 && mcc === 1,
      leafExists: (pair, leaf) => pair === 0 && leaf === "X",
      nodeExists: (pair, node) => pair === 0 && node.side === "left" && node.name === "NODE_1",
    });

    expect(resolveWorkspaceSearch(search, present)).toStrictEqual({
      ...DEFAULT_RESOLVED,
      view: "tanglegram",
      ...selection,
    });
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
      { ...DEFAULT_SEARCH, pair: ["ha", "pb2"], mcc: 1, leaf: "X", node: { side: "arg", name: "H" } },
      recording,
    );

    expect(checkedPairs).toStrictEqual([0, 0, 0]);
  });

  test("keeps the Auspice value", () => {
    const search: WrittenSearch = { ...DEFAULT_SEARCH, view: "auspice", auspice: "c=imputed" };

    expect(resolveWorkspaceSearch(search, TWO_TREE_RESULT)).toStrictEqual({
      ...DEFAULT_RESOLVED,
      view: "auspice",
      auspice: "c=imputed",
    });
  });
});

describe("selectPair", () => {
  test("writes the labels of the pair and clears the mcc and the node, keeping the leaf", () => {
    const search: WrittenSearch = {
      ...DEFAULT_SEARCH,
      view: "tanglegram",
      mcc: 2,
      leaf: "X",
      node: { side: "right", name: "NODE_4" },
    };

    expect(selectPair(search, THREE_TREE_PAIRS, 1)).toStrictEqual({
      ...DEFAULT_SEARCH,
      view: "tanglegram",
      pair: ["ha", "pb1"],
      leaf: "X",
    });
  });

  test("writes no pair for the first pair, the default", () => {
    expect(selectPair({ ...DEFAULT_SEARCH, pair: ["na", "pb1"] }, THREE_TREE_PAIRS, 0)).toStrictEqual(DEFAULT_SEARCH);
  });

  test("drops the kept leaf when the new pair lacks it", () => {
    const leafOnlyInPairZero = availability({ ...THREE_TREE_RESULT, leafExists: (pair) => pair === 0 });

    const resolved = resolveWorkspaceSearch(
      selectPair({ ...DEFAULT_SEARCH, leaf: "X" }, THREE_TREE_PAIRS, 2),
      leafOnlyInPairZero,
    );

    expect(resolved).toStrictEqual({ ...DEFAULT_RESOLVED, pair: 2 });
  });
});

describe("withSelectionPair", () => {
  const HIDDEN_PAIR: WrittenSearch = { ...DEFAULT_SEARCH, view: "tanglegram", pair: ["ha", "pb2"] };

  test("writes the shown pair with a selection made in it, so the selection and its pair agree", () => {
    expect(withSelectionPair(HIDDEN_PAIR, { ...HIDDEN_PAIR, mcc: 4 }, THREE_TREE_PAIRS, 1)).toStrictEqual({
      ...HIDDEN_PAIR,
      pair: ["ha", "pb1"],
      mcc: 4,
    });
  });

  test("writes no pair when the selection is made in the first pair", () => {
    expect(withSelectionPair(HIDDEN_PAIR, { ...HIDDEN_PAIR, mcc: 4 }, THREE_TREE_PAIRS, 0)).toStrictEqual({
      ...DEFAULT_SEARCH,
      view: "tanglegram",
      mcc: 4,
    });
  });

  test("writes the shown pair when a leaf or node selection changes or is cleared", () => {
    const node: NodeRef = { side: "left", name: "NODE_1" };
    const shown: PairRef = ["na", "pb1"];

    expect({
      leaf: withSelectionPair(HIDDEN_PAIR, { ...HIDDEN_PAIR, leaf: "A" }, THREE_TREE_PAIRS, 2).pair,
      node: withSelectionPair(HIDDEN_PAIR, { ...HIDDEN_PAIR, node }, THREE_TREE_PAIRS, 2).pair,
      cleared: withSelectionPair({ ...HIDDEN_PAIR, mcc: 1 }, HIDDEN_PAIR, THREE_TREE_PAIRS, 2).pair,
    }).toStrictEqual({ leaf: shown, node: shown, cleared: shown });
  });

  test("keeps the written pair when the selection does not change", () => {
    const next: WrittenSearch = { ...HIDDEN_PAIR, mcc: 1, scale: "depth" };

    expect(withSelectionPair({ ...HIDDEN_PAIR, mcc: 1 }, next, THREE_TREE_PAIRS, 0)).toStrictEqual(next);
  });

  test("keeps a pair that the change itself selects", () => {
    const next: WrittenSearch = { ...HIDDEN_PAIR, pair: ["na", "pb1"], leaf: "A" };

    expect(withSelectionPair(HIDDEN_PAIR, next, THREE_TREE_PAIRS, 0)).toStrictEqual(next);
  });
});

describe("searchAfterRun", () => {
  const SHOWN: Omit<WrittenSearch, "view"> = {
    pair: ["ha", "pb1"],
    version: "imputed",
    scale: "depth",
    labels: "off",
    leaf: "A",
  };

  const SUMMARY: Summary = {
    pairs: [],
    arg: { status: "built", reassortments: 1 },
    noReassortment: false,
    diagnostics: [],
  };

  const SUCCEEDED: RunOutcome = { status: "succeeded", sessionId: 7, summary: SUMMARY };

  test.each(["overview", "files", "tanglegram"] as const)(
    "opens the Auspice view from the %s view for a stored result, keeping pair, version, and scale",
    (view) => {
      expect(searchAfterRun({ ...SHOWN, view }, SUCCEEDED, 7, false)).toStrictEqual({ ...SHOWN, view: "auspice" });
    },
  );

  test("keeps the view that a link names for the run it starts", () => {
    expect(searchAfterRun({ ...SHOWN, view: "mccs" }, SUCCEEDED, 7, true)).toStrictEqual({ ...SHOWN, view: "mccs" });
  });

  test.each<{ run: string; outcome: RunOutcome; stored: number | undefined }>([
    { run: "a cancelled", outcome: { status: "failed", kind: "cancelled", message: "Run cancelled." }, stored: 7 },
    { run: "a failed", outcome: { status: "failed", kind: "internal", message: "Out of memory." }, stored: 7 },
    { run: "an invalid", outcome: { status: "failed", kind: "invalid", message: "No trees." }, stored: 7 },
    { run: "a superseded", outcome: SUCCEEDED, stored: 8 },
    { run: "a discarded", outcome: SUCCEEDED, stored: undefined },
  ])("leaves the search unchanged after $run run", ({ outcome, stored }) => {
    const search: WrittenSearch = { ...SHOWN, view: "files" };

    expect(searchAfterRun(search, outcome, stored, false)).toStrictEqual(search);
  });
});

function parseUrl(query: string): WrittenSearch {
  return workspaceSearchSchema.parse(parseSearch(query));
}

function availability(overrides: Partial<WorkspaceAvailability>): WorkspaceAvailability {
  return { ...NO_WORKSPACE, ...overrides };
}
