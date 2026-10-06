import { describe, expect, test } from "vitest";

import { WORKSPACE_SEARCH_DEFAULTS, type WorkspaceSearch, type WrittenSearch } from "../../workspace/search";
import {
  argClickSelection,
  argEmphasis,
  argSelectionRows,
  emphasisOpacity,
  NO_SELECTION,
  pairClickSelection,
  pairEmphasis,
  pairSelectionRows,
  type Selection,
  selectionOf,
  UNSELECTED_OPACITY,
  withSelection,
} from "../selection";
import { exampleArgView, examplePairView } from "./fixtures";

const VIEW = examplePairView();

const ARG = exampleArgView();

const SEARCH: WrittenSearch = { ...WORKSPACE_SEARCH_DEFAULTS, view: "tanglegram", mcc: 1, leaf: "A" };

const RESOLVED: WorkspaceSearch = { ...SEARCH, pair: 0, show: "both" };

describe("pairClickSelection", () => {
  test.each([
    ["the branch of leaf C", { kind: "node", side: "left", node: 5 }, { leaf: "C" }],
    ["the tip of leaf X in the right tree", { kind: "node", side: "right", node: 5 }, { leaf: "X" }],
    ["the link of X", { kind: "link", link: 4 }, { mcc: 1 }],
    ["the ribbon of C and D", { kind: "ribbon", block: 1 }, { mcc: 0 }],
    [
      "the branch above the clade of A and B",
      { kind: "node", side: "left", node: 1 },
      { mcc: 0, node: { side: "left", name: "NODE_2" } },
    ],
    [
      "an internal branch without an MCC",
      { kind: "node", side: "right", node: 1 },
      { node: { side: "right", name: "NODE_2" } },
    ],
    ["the background", undefined, NO_SELECTION],
  ] as const)("a click on %s selects the right values", (_, target, expected) => {
    expect(pairClickSelection(VIEW, target)).toStrictEqual(expected);
  });
});

describe("argClickSelection", () => {
  test.each([
    ["leaf B", { kind: "node", node: 3 }, { leaf: "B" }],
    ["the hybrid node", { kind: "node", node: 4 }, { node: { side: "arg", name: "H" } }],
    ["the reticulation edge into the hybrid node", { kind: "edge", edge: 4 }, { node: { side: "arg", name: "H" } }],
    ["the internal node N1", { kind: "node", node: 1 }, { node: { side: "arg", name: "N1" } }],
    ["the background", undefined, NO_SELECTION],
  ] as const)("a click on %s selects the right values", (_, target, expected) => {
    expect(argClickSelection(ARG, target)).toStrictEqual(expected);
  });
});

describe("withSelection", () => {
  test("replaces the whole selection and keeps the other params", () => {
    expect(withSelection(SEARCH, { node: { side: "left", name: "NODE_2" } })).toStrictEqual({
      ...WORKSPACE_SEARCH_DEFAULTS,
      view: "tanglegram",
      node: { side: "left", name: "NODE_2" },
    });
  });

  test("round-trips the selection of a search", () => {
    expect(withSelection({ ...WORKSPACE_SEARCH_DEFAULTS }, selectionOf(RESOLVED))).toStrictEqual({
      ...WORKSPACE_SEARCH_DEFAULTS,
      mcc: 1,
      leaf: "A",
    });
  });
});

describe("pairEmphasis", () => {
  test("finds the selected leaf in both trees and its link", () => {
    expect(pairEmphasis(VIEW, { leaf: "X" })).toStrictEqual({
      mcc: undefined,
      leaf: { left: 8, right: 5, link: 4 },
      node: undefined,
    });
  });

  test("finds a selected node on its side", () => {
    expect(pairEmphasis(VIEW, { mcc: 0, node: { side: "right", name: "NODE_3" } })).toStrictEqual({
      mcc: 0,
      leaf: { left: undefined, right: undefined, link: undefined },
      node: { side: "right", node: 6 },
    });
  });

  test("ignores a node selection whose name is a leaf of the tree", () => {
    expect(pairEmphasis(VIEW, { node: { side: "right", name: "X" } }).node).toBeUndefined();
  });

  test("ignores an ARG node", () => {
    expect(pairEmphasis(VIEW, { node: { side: "arg", name: "NODE_3" } }).node).toBeUndefined();
  });
});

describe("argEmphasis", () => {
  test("finds the selected leaf and node", () => {
    expect(argEmphasis(ARG, { leaf: "C", node: { side: "arg", name: "H" } })).toStrictEqual({ leaf: 5, node: 4 });
  });

  test("ignores a node of a tree", () => {
    expect(argEmphasis(ARG, { node: { side: "left", name: "H" } })).toStrictEqual({
      leaf: undefined,
      node: undefined,
    });
  });
});

describe("emphasisOpacity", () => {
  test.each([
    ["nothing selected", 0, undefined, 1],
    ["the selected MCC", 1, 1, 1],
    ["another MCC", 0, 1, UNSELECTED_OPACITY],
    ["a node without an MCC", null, 1, UNSELECTED_OPACITY],
  ] as const)("%s draws at its opacity", (_, mcc, selected, opacity) => {
    expect(emphasisOpacity(mcc, selected)).toBe(opacity);
  });
});

describe("pairSelectionRows", () => {
  test.each([
    [
      "the rows of the leaves under a selected node",
      { mcc: 0, node: { side: "left", name: "NODE_2" } },
      { first: 0, last: 1 },
    ],
    ["the rows of both copies of a selected leaf", { mcc: 0, leaf: "X" }, { first: 2, last: 4 }],
    ["the rows of the leaves of a selected MCC in both trees", { mcc: 0 }, { first: 0, last: 4 }],
    ["the MCC rows when the selected leaf is not drawn", { mcc: 1, leaf: "Z" }, { first: 2, last: 4 }],
    ["nothing without a selection", {}, null],
  ] satisfies [string, Selection, unknown][])("zooms to %s", (_case, selection, expected) => {
    expect(pairSelectionRows(VIEW, selection)).toStrictEqual(expected);
  });
});

describe("argSelectionRows", () => {
  test.each([
    ["the leaves under a selected node", { node: { side: "arg", name: "N1" } }, { first: 0, last: 1 }],
    ["every leaf under the root", { node: { side: "arg", name: "ROOT" } }, { first: 0, last: 2 }],
    ["the row of a selected leaf", { leaf: "C" }, { first: 2, last: 2 }],
    ["nothing without a selection", {}, null],
  ] satisfies [string, Selection, unknown][])("zooms to %s", (_case, selection, expected) => {
    expect(argSelectionRows(ARG, selection)).toStrictEqual(expected);
  });
});
