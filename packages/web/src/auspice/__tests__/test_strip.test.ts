import type { PairSummary } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { shownFromStrip, shownTreeIndices } from "../strip";

const PAIRS: PairSummary[] = [
  pairOf(0, [0, 1]),
  pairOf(1, [0, 2]),
  pairOf(2, [0, 3]),
  pairOf(3, [1, 2]),
  pairOf(4, [1, 3]),
  pairOf(5, [2, 3]),
];

describe("shownTreeIndices", () => {
  test.each([
    ["both", [1, 3]],
    ["left", [1]],
    ["right", [3]],
  ] as const)("the strip of pair (1, 3) with %s shown turns on trees %j", (trees, indices) => {
    expect(shownTreeIndices(pairOf(4, [1, 3]), trees)).toStrictEqual(indices);
  });
});

describe("shownFromStrip", () => {
  test.each([
    ["two trees in run order", [0, 2], { pair: 1, trees: "both" }],
    ["two trees in the order of their toggles", [3, 1], { pair: 4, trees: "both" }],
    ["the first tree of the pair alone", [1], { pair: 4, trees: "left" }],
    ["the second tree of the pair alone", [3], { pair: 4, trees: "right" }],
  ] as const)("%s", (_case, selected, shown) => {
    expect(shownFromStrip(selected, PAIRS, { pair: 4, trees: "both" })).toStrictEqual(shown);
  });

  test.each([
    ["no tree", []],
    ["three trees", [0, 1, 2]],
    ["one tree outside the shown pair", [0]],
  ] as const)("changes nothing for %s", (_case, selected) => {
    expect(shownFromStrip(selected, PAIRS, { pair: 4, trees: "both" })).toBeNull();
  });

  test("maps the strip of the shown trees back to the same choice", () => {
    const current = { pair: 3, trees: "right" } as const;

    expect(shownFromStrip(shownTreeIndices(pairOf(3, [1, 2]), "right"), PAIRS, current)).toStrictEqual(current);
  });
});

function pairOf(index: number, trees: [number, number]): PairSummary {
  return {
    index,
    trees,
    labels: [`t${String(trees[0])}`, `t${String(trees[1])}`],
    mccCount: 0,
    mccs: [],
    imputedCount: 0,
    ambiguousCount: 0,
  };
}
