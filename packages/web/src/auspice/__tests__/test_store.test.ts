import type { AuspiceTrees } from "@neherlab/treeknit-wasm";
import { applyFilter } from "auspice/src/actions/tree";
import { CLEAN_START, DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import { strainSymbol } from "auspice/src/util/globals";
import type { AnyAction } from "redux";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import type { Selection } from "../../drawing/selection";
import { mccFilterValue, NO_MARKS, syncSelection, zoomToMcc } from "../linking";
import { auspiceQuery, parseAuspiceQuery } from "../query";
import { createAuspiceStore } from "../store";
import { loadAuspiceStore } from "../useAuspiceStore";
import { AUSPICE_LABELS, AUSPICE_PAIR } from "./auspicePair";

describe("auspice store", () => {
  beforeEach(() => {
    vi.stubGlobal("window", { innerWidth: 1280, innerHeight: 800, document: { body: { clientHeight: 800 } } });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test.each([
    { tips: 4000, skipTreeAnimation: false },
    { tips: 4001, skipTreeAnimation: true },
  ])(
    "a tree of $tips tips sets skipTreeAnimation to $skipTreeAnimation (auspice performanceFlags: above 4000 tips)",
    ({ tips, skipTreeAnimation }) => {
      const store = createAuspiceStore();

      store.dispatch({
        type: CLEAN_START,
        metadata: { loaded: true },
        tree: { loaded: true, nodes: [{ fullTipCount: tips }] },
        controls: {},
        entropy: {},
        measurements: {},
      });

      expect(store.getState().controls.performanceFlags).toStrictEqual(
        new Map([["skipTreeAnimation", skipTreeAnimation]]),
      );
    },
  );

  test("a pair shows the right tree next to the left one, rectangular, colored by MCC, with the legend closed and one line per shared tip", () => {
    const { store } = loadedPair();
    const { controls, tree, treeToo } = store.getState();

    expect({
      mainTree: tree.name,
      showTreeToo: controls.showTreeToo,
      showTangle: controls.showTangle,
      layout: controls.layout,
      colorBy: controls.colorBy,
      legendOpen: controls.legendOpen,
      tangleLines: treeToo.tangleTipLookup?.length,
    }).toStrictEqual({
      mainTree: "a",
      showTreeToo: "b",
      showTangle: true,
      layout: "rect",
      colorBy: "mcc",
      legendOpen: false,
      tangleLines: 5,
    });
  });

  test("building the store leaves the datasets unchanged", () => {
    const before = structuredClone(AUSPICE_PAIR);

    loadedPair();

    expect(AUSPICE_PAIR).toStrictEqual(before);
  });

  test.each<{ action: string; dispatched: (names: NodeIndex) => AnyAction; expected: Selection }>([
    {
      action: "a tip click",
      dispatched: ({ left }) => ({ type: SELECT_NODE, name: "X", idx: left("X"), isBranch: false, treeId: "LEFT" }),
      expected: { leaf: "X" },
    },
    {
      action: "a click on the branch above a tip of the right tree",
      dispatched: ({ right }) => ({ type: SELECT_NODE, name: "B", idx: right("B"), isBranch: true, treeId: "RIGHT" }),
      expected: { leaf: "B" },
    },
    {
      action: "a branch of the left tree",
      dispatched: ({ left }) => ({
        type: SELECT_NODE,
        name: "NODE_4",
        idx: left("NODE_4"),
        isBranch: true,
        treeId: "LEFT",
      }),
      expected: { node: { side: "left", name: "NODE_4" } },
    },
    {
      action: "a branch of the right tree",
      dispatched: ({ right }) => ({
        type: SELECT_NODE,
        name: "NODE_3",
        idx: right("NODE_3"),
        isBranch: true,
        treeId: "RIGHT",
      }),
      expected: { node: { side: "right", name: "NODE_3" } },
    },
    { action: "a deselection", dispatched: () => ({ type: DESELECT_NODE }), expected: {} },
  ])("$action selects $expected in the workspace", ({ dispatched, expected }) => {
    const { store, selections } = loadedPair();

    store.dispatch(dispatched(nodeIndex(store.getState())));

    expect(selections).toStrictEqual([expected]);
  });

  test("a tip click keeps Auspice's own state of the selected node", () => {
    const { store } = loadedPair();
    const idx = nodeIndex(store.getState()).left("A");

    store.dispatch({ type: SELECT_NODE, name: "A", idx, isBranch: false, treeId: "LEFT" });

    expect(store.getState().controls.selectedNode).toMatchObject({ name: "A", idx, isBranch: false, treeId: "LEFT" });
  });

  test("other actions select nothing in the workspace", () => {
    const { store, selections } = loadedPair();

    store.dispatch({ type: "TOGGLE_TANGLE" });

    expect(selections).toStrictEqual([]);
  });
});

describe("auspice store of one tree", () => {
  beforeEach(() => {
    vi.stubGlobal("window", { innerWidth: 1280, innerHeight: 800, document: { body: { clientHeight: 800 } } });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test("shows the right tree of the pair alone as the main tree", () => {
    const { store } = loadedPair("right");
    const { controls, tree } = store.getState();

    expect({ mainTree: tree.name, showTreeToo: controls.showTreeToo }).toStrictEqual({
      mainTree: "b",
      showTreeToo: false,
    });
  });

  test("reports a branch of the right tree shown alone on the right side", () => {
    const { store, selections } = loadedPair("right");
    const idx = nodeIndex(store.getState()).left("NODE_3");

    store.dispatch({ type: SELECT_NODE, name: "NODE_3", idx, isBranch: true, treeId: "LEFT" });

    expect(selections).toStrictEqual([{ node: { side: "right", name: "NODE_3" } }]);
  });
});

describe("auspice links with the workspace", () => {
  beforeEach(() => {
    vi.stubGlobal("window", { innerWidth: 1280, innerHeight: 800, document: { body: { clientHeight: 800 } } });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test("reads its choices from the URL query and writes the same query back", () => {
    const text = "c=largest_mcc&branchLabel=none&showBranchLabels=all&legend=open&focus=selected&f_mcc=MCC+2";
    const { store } = loadedPair("both", parseAuspiceQuery(text));

    expect(auspiceQuery(store.getState().controls)).toBe(text);
  });

  test("writes the layout of one tree, and nothing for the dataset defaults", () => {
    const radial = loadedPair("left", parseAuspiceQuery("l=radial")).store;
    const plain = loadedPair("left").store;

    expect([auspiceQuery(radial.getState().controls), auspiceQuery(plain.getState().controls)]).toStrictEqual([
      "l=radial",
      "",
    ]);
  });

  test("leaves the strain filter of a selected tip out of the query, and keeps a filter that existed before", () => {
    const { store } = loadedPair("both", parseAuspiceQuery("s=B"));
    const index = nodeIndex(store.getState());

    store.dispatch({ type: SELECT_NODE, name: "A", idx: index.left("A"), isBranch: false, treeId: "LEFT" });
    store.dispatch(applyFilter("add", strainSymbol, ["A"]));
    const withA = auspiceQuery(store.getState().controls);

    store.dispatch({ type: SELECT_NODE, name: "B", idx: index.left("B"), isBranch: false, treeId: "LEFT" });

    expect([withA, auspiceQuery(store.getState().controls)]).toStrictEqual(["s=B", "s=B%2CA"]);
  });

  test("reports each change of its choices once", () => {
    const { store, queries } = loadedPair();

    store.dispatch(applyFilter("add", "mcc", ["MCC 1"]));
    store.dispatch({ type: "TOGGLE_TANGLE" });

    expect(queries.slice(1)).toStrictEqual(["f_mcc=MCC+1"]);
  });

  test("marks a leaf of the workspace selection as auspice marks a clicked tip, and removes the marks with the selection", () => {
    const { store, selections } = loadedPair();
    const marks = syncSelection(store, NO_MARKS, { leaf: "C" }, (mcc) => mccFilterValue(AUSPICE_PAIR.left, mcc));
    const marked = store.getState().controls;
    const markedState = { selected: marked.selectedNode?.name, strain: marked.filters[strainSymbol] };

    syncSelection(store, marks, {}, (mcc) => mccFilterValue(AUSPICE_PAIR.left, mcc));
    const cleared = store.getState().controls;

    expect({
      markedState,
      cleared: { selected: cleared.selectedNode, strain: cleared.filters[strainSymbol] },
      selections,
    }).toStrictEqual({
      markedState: { selected: "C", strain: [{ value: "C", active: true }] },
      cleared: { selected: null, strain: undefined },
      selections: [],
    });
  });

  test("filters to a selected MCC and removes only that filter value afterwards", () => {
    const { store } = loadedPair();
    const value = (mcc: number) => mccFilterValue(AUSPICE_PAIR.left, mcc);
    const marks = syncSelection(store, NO_MARKS, { mcc: 1 }, value);
    const filtered = store.getState().controls.filters["mcc"];

    syncSelection(store, marks, {}, value);

    expect({ filtered, cleared: store.getState().controls.filters["mcc"] }).toStrictEqual({
      filtered: [{ value: "MCC 2", active: true }],
      cleared: undefined,
    });
  });

  test("adopts a tip that auspice selected without marking it again", () => {
    const { store } = loadedPair();
    const idx = nodeIndex(store.getState()).left("A");

    store.dispatch({ type: SELECT_NODE, name: "A", idx, isBranch: false, treeId: "LEFT" });
    store.dispatch(applyFilter("add", strainSymbol, ["A"]));

    const marks = syncSelection(store, NO_MARKS, { leaf: "A" }, () => undefined);

    expect({ marks, strain: store.getState().controls.filters[strainSymbol] }).toStrictEqual({
      marks: { leaf: { name: "A", existingFilter: null }, mcc: null },
      strain: [{ value: "A", active: true }],
    });
  });

  test("zooms each tree to the node where an MCC starts", () => {
    const { store } = loadedPair();
    const index = nodeIndex(store.getState());
    const root = AUSPICE_PAIR.mcc_roots[1];

    if (root === undefined) {
      throw new Error("the fixture has two MCCs");
    }

    zoomToMcc(store, { left: "NODE_4", right: "NODE_3" }, "both");

    expect([store.getState().tree.idxOfInViewRootNode, store.getState().treeToo.idxOfInViewRootNode]).toStrictEqual([
      index.left("NODE_4"),
      index.right("NODE_3"),
    ]);
  });
});

describe("mccFilterValue", () => {
  test("takes the value of an MCC from the coloring of the dataset", () => {
    expect([
      mccFilterValue(AUSPICE_PAIR.left, 0),
      mccFilterValue(AUSPICE_PAIR.left, 1),
      mccFilterValue(AUSPICE_PAIR.left, 2),
    ]).toStrictEqual(["MCC 1", "MCC 2", undefined]);
  });
});

interface NodeIndex {
  left: (name: string) => number;
  right: (name: string) => number;
}

function loadedPair(trees: AuspiceTrees = "both", query: Record<string, string> = {}) {
  const selections: Selection[] = [];
  const queries: string[] = [];

  const store = loadAuspiceStore({
    datasets: AUSPICE_PAIR,
    labels: AUSPICE_LABELS,
    trees,
    query,
    onSelect: (selection) => {
      selections.push(selection);
    },
    onQuery: (text) => {
      queries.push(text);
    },
  });

  return { store, selections, queries };
}

function nodeIndex(state: ReturnType<ReturnType<typeof createAuspiceStore>["getState"]>): NodeIndex {
  const find = (nodes: readonly { name: string; arrayIdx: number }[] | null | undefined) => (name: string) => {
    const node = nodes?.find((candidate) => candidate.name === name);

    if (node === undefined) {
      throw new Error(`no node ${name}`);
    }

    return node.arrayIdx;
  };

  return { left: find(state.tree.nodes), right: find(state.treeToo.nodes) };
}
