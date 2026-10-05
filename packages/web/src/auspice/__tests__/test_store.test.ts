import { CLEAN_START, DESELECT_NODE, SELECT_NODE } from "auspice/src/actions/types";
import type { AnyAction } from "redux";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import type { Selection } from "../../drawing/selection";
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

  test("a pair shows the right tree next to the left one, rectangular, colored by MCC, with one line per shared tip", () => {
    const { store } = loadedPair();
    const { controls, tree, treeToo } = store.getState();

    expect({
      mainTree: tree.name,
      showTreeToo: controls.showTreeToo,
      showTangle: controls.showTangle,
      layout: controls.layout,
      colorBy: controls.colorBy,
      tangleLines: treeToo.tangleTipLookup?.length,
    }).toStrictEqual({
      mainTree: "a",
      showTreeToo: "b",
      showTangle: true,
      layout: "rect",
      colorBy: "mcc",
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

interface NodeIndex {
  left: (name: string) => number;
  right: (name: string) => number;
}

function loadedPair() {
  const selections: Selection[] = [];

  const store = loadAuspiceStore({
    datasets: AUSPICE_PAIR,
    labels: AUSPICE_LABELS,
    onSelect: (selection) => {
      selections.push(selection);
    },
  });

  return { store, selections };
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
