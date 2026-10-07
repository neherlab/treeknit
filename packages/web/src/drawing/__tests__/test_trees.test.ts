import { describe, expect, test } from "vitest";

import {
  argLeafNames,
  argLeafRows,
  argNodePoints,
  internalNodeIndex,
  leafIndex,
  leafNames,
  nodeRows,
  pairLeafNames,
  pairLeafRows,
  rowCount,
  treeNodePoints,
} from "../trees";
import { exampleArgView, examplePairView } from "./fixtures";

const VIEW = examplePairView();

describe("treeNodePoints", () => {
  test("places each node at the end of its elbow and the root at the start of its children's elbows", () => {
    const points = treeNodePoints(VIEW.left, VIEW.shapes.left.elbows);

    expect(points).toStrictEqual([
      [0, 2],
      [0.5, 0.5],
      [1, 0],
      [1, 1],
      [0.25, 3],
      [1, 2],
      [0.5, 3.5],
      [1, 3],
      [1, 4],
    ]);
  });
});

describe("argNodePoints", () => {
  test("reads node positions from elbows and reticulation curves", () => {
    expect(argNodePoints(exampleArgView())).toStrictEqual([
      [0, 1.5],
      [0.5, 0.5],
      [1, 0],
      [1, 1],
      [0.75, 2.5],
      [1, 2],
    ]);
  });
});

describe("nodeRows", () => {
  test("covers the rows of every given node and skips missing nodes", () => {
    expect(nodeRows({ rows: { first: 3, last: 5 } }, undefined, { rows: { first: 1, last: 1 } })).toStrictEqual({
      first: 1,
      last: 5,
    });
  });

  test("gives no range without a node", () => {
    expect([nodeRows(), nodeRows(undefined)]).toStrictEqual([null, null]);
  });
});

describe("node indices", () => {
  const renamed = new Map([
    [0, ""],
    [1, "A"],
    [4, ""],
  ]);

  const tree = {
    ...VIEW.left,
    nodes: VIEW.left.nodes.map((node, index) => ({ ...node, name: renamed.get(index) ?? node.name })),
  };

  test("finds a leaf and an internal node of the same name separately", () => {
    expect({ leaf: leafIndex(tree, "A"), internal: internalNodeIndex(tree, "A") }).toStrictEqual({
      leaf: 2,
      internal: 1,
    });
  });

  test("finds the first of two nodes of one name, and nothing for an unknown name", () => {
    expect({ unnamed: internalNodeIndex(tree, ""), unknown: leafIndex(tree, "Z") }).toStrictEqual({
      unnamed: 0,
      unknown: undefined,
    });
  });
});

describe("tree lookups", () => {
  test("lists the leaf names in display order", () => {
    expect(leafNames(VIEW.right)).toStrictEqual(["A", "B", "X", "C", "D"]);
  });

  test("spans the rows of a leaf's copies in both trees", () => {
    expect({
      both: pairLeafRows(VIEW, "X"),
      same: pairLeafRows(VIEW, "A"),
      none: pairLeafRows(VIEW, "nope"),
    }).toStrictEqual({ both: { first: 2, last: 4 }, same: { first: 0, last: 0 }, none: null });
  });

  test("lists the leaf names of both trees once", () => {
    expect(pairLeafNames(VIEW)).toStrictEqual(["A", "B", "C", "D", "X"]);
  });

  test("lists the ARG's leaves and finds the row of one", () => {
    const arg = exampleArgView();

    expect({ names: argLeafNames(arg), c: argLeafRows(arg, "C"), none: argLeafRows(arg, "Z") }).toStrictEqual({
      names: ["A", "B", "C"],
      c: { first: 2, last: 2 },
      none: null,
    });
  });

  test("counts the rows of the taller tree", () => {
    expect(rowCount(VIEW.left, VIEW.right)).toBe(5);
  });
});
