import type { Leader } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { columnPixel } from "../../canvas/projection";
import { examplePairView } from "../../drawing/__tests__/fixtures";
import { tanglegramColumns } from "../columns";
import { PAIR_LAYER, pairTargetAt } from "../geometry";
import { tanglegramGeometry } from "./geometry";

const VIEW = examplePairView();

const COLUMNS = tanglegramColumns(1232, 100);

const GEOMETRY = tanglegramGeometry(VIEW, COLUMNS, "y", 64);

describe("tanglegramGeometry", () => {
  test("maps the elbows of the left tree into its column", () => {
    const branch = GEOMETRY.branches.plain.find((item) => item.side === "left" && item.node === 2);

    expect(branch?.path).toStrictEqual([
      [221, 0.5],
      [221, 0],
      [426, 0],
    ]);
  });

  test("mirrors the right tree: its root sits at the right edge", () => {
    expect(GEOMETRY.nodes.right[0]).toStrictEqual([COLUMNS.right.end, 2]);
  });

  test("sorts branches by kind: reassortment wins over added and plain", () => {
    const kinds = {
      plain: GEOMETRY.branches.plain.map(({ side, node }) => `${side}${String(node)}`),
      added: GEOMETRY.branches.added.map(({ side, node }) => `${side}${String(node)}`),
      reassortment: GEOMETRY.branches.reassortment.map(({ side, node }) => `${side}${String(node)}`),
    };

    expect(kinds).toStrictEqual({
      plain: ["left2", "left3", "left4", "left6", "right1", "right7", "right8"],
      added: ["right3"],
      reassortment: ["left1", "left5", "left7", "left8", "right2", "right4", "right5", "right6"],
    });
  });

  test("starts each link at the left edge of the link zone and ends it at the right edge", () => {
    const link = GEOMETRY.links[4];

    expect([link?.path[0], link?.path.at(-1), link?.mcc]).toStrictEqual([
      [COLUMNS.links.start, 4],
      [COLUMNS.links.end, 2],
      1,
    ]);
  });

  test("closes each ribbon outline and keeps the block order", () => {
    expect(
      GEOMETRY.ribbons.map(({ block, mcc, polygon }) => ({
        block,
        mcc,
        closed: JSON.stringify(polygon[0]) === JSON.stringify(polygon.at(-1)),
        first: polygon[0],
      })),
    ).toStrictEqual([
      { block: 0, mcc: 0, closed: true, first: [COLUMNS.links.start, -0.5] },
      { block: 1, mcc: 0, closed: true, first: [COLUMNS.links.start, 1.5] },
      { block: 2, mcc: 1, closed: true, first: [COLUMNS.links.start, 3.5] },
    ]);
  });

  test("runs a leader from the leaf tip to the label edge of its tree column, mirrored on the right", () => {
    const leader: Leader = { node: 2, from: [0.5, 0], to: [1, 0] };

    const view = {
      ...VIEW,
      // oxlint-disable-next-line anti-slop/no-shape-in-symbol-names -- field name of the generated PairView type
      shapes: {
        ...VIEW.shapes,
        left: { ...VIEW.shapes.left, leaders: [leader] },
        right: { ...VIEW.shapes.right, leaders: [leader] },
      },
    };

    expect(tanglegramGeometry(view, COLUMNS, "y", 64).leaders).toStrictEqual([
      {
        side: "left",
        node: 2,
        mcc: VIEW.left.nodes[2]?.mcc,
        path: [
          [columnPixel(COLUMNS.left, 0.5), 0],
          [COLUMNS.left.end, 0],
        ],
      },
      {
        side: "right",
        node: 2,
        mcc: VIEW.right.nodes[2]?.mcc,
        path: [
          [columnPixel(COLUMNS.right, 0.5), 0],
          [COLUMNS.right.start, 0],
        ],
      },
    ]);
  });

  test("puts the left labels at the start of their column and the right labels at the end of theirs", () => {
    expect([GEOMETRY.labels.left[0]?.position, GEOMETRY.labels.right[0]?.position]).toStrictEqual([
      [COLUMNS.leftLabels.start, 0],
      [COLUMNS.rightLabels.end, 0],
    ]);
  });

  test("swaps the axes on a narrow canvas", () => {
    const columns = tanglegramColumns(400, 0);
    const narrow = tanglegramGeometry(VIEW, columns, "x", 64);

    expect(narrow.nodes.left[8]).toStrictEqual([4, columnPixel(columns.left, 1)]);
  });
});

describe("pairTargetAt", () => {
  test.each([
    [PAIR_LAYER.links, 4, { kind: "link", link: 4 }],
    [PAIR_LAYER.ribbons, 2, { kind: "ribbon", block: 2 }],
    [PAIR_LAYER.addedBranches, 0, { kind: "node", side: "right", node: 3 }],
    [PAIR_LAYER.rightLabels, 2, { kind: "node", side: "right", node: 5 }],
    [PAIR_LAYER.imputedMarks, 0, { kind: "node", side: "right", node: 5 }],
    ["minimap-window", 0, undefined],
    [PAIR_LAYER.links, 99, undefined],
  ] as const)("a pick on %s at %i targets %o", (layer, index, target) => {
    expect(pairTargetAt(GEOMETRY, layer, index)).toStrictEqual(target);
  });
});
