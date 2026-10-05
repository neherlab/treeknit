import { describe, expect, test } from "vitest";

import { argNodePoints, cladeSize, leafNames, leafRow, leafRows, rowCount, treeNodePoints } from "../trees";
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

describe("leafRows", () => {
  test.each([
    ["the root spans every row", 0, { first: 0, last: 4 }],
    ["a cherry spans its two leaves", 1, { first: 0, last: 1 }],
    ["a leaf spans its own row", 8, { first: 4, last: 4 }],
  ] as const)("%s", (_, node, expected) => {
    expect(leafRows(VIEW.left.nodes, node)).toStrictEqual(expected);
  });

  test("follows both parents of a hybrid node without visiting it twice", () => {
    expect(leafRows(exampleArgView().nodes, 0)).toStrictEqual({ first: 0, last: 2 });
  });

  test("spans a caterpillar tree deeper than a function call takes arguments", () => {
    const leaves = 300_000;
    const nodes = caterpillar(leaves);

    expect(leafRows(nodes, 0)).toStrictEqual({ first: 0, last: leaves - 1 });
  });

  test("spans a polytomy with more children than a function call takes arguments", () => {
    const leaves = 300_000;
    const children = Array.from({ length: leaves }, (_, index) => index + 1);

    const nodes = [
      { children, leaf: false, y: 0 },
      ...children.map((_, row) => ({ children: [], leaf: true, y: row })),
    ];

    expect(leafRows(nodes, 0)).toStrictEqual({ first: 0, last: leaves - 1 });
  });

  test("gives no range for a node outside the tree", () => {
    expect(leafRows(VIEW.left.nodes, 99)).toBeNull();
  });
});

describe("tree lookups", () => {
  test("counts the leaves of a clade", () => {
    expect(cladeSize(VIEW.left.nodes, 4)).toBe(3);
  });

  test("lists the leaf names in display order", () => {
    expect(leafNames(VIEW.right)).toStrictEqual(["A", "B", "X", "C", "D"]);
  });

  test("finds the row of a leaf in the left tree first", () => {
    expect(leafRow(VIEW.left, VIEW.right, "X")).toBe(4);
  });

  test("counts the rows of the taller tree", () => {
    expect(rowCount(VIEW.left, VIEW.right)).toBe(5);
  });
});

function caterpillar(leaves: number) {
  const nodes: { children: number[]; leaf: boolean; y: number }[] = [];

  for (let row = 0; row < leaves - 1; row += 1) {
    const inner = nodes.length;

    nodes.push({ children: [inner + 1, inner + 2], leaf: false, y: row }, { children: [], leaf: true, y: row });
  }

  nodes.push({ children: [], leaf: true, y: leaves - 1 });

  return nodes;
}
