import { describe, expect, test } from "vitest";

import { leafNames, shownTreeLabels } from "../datasets";
import { AUSPICE_LABELS, AUSPICE_PAIR } from "./auspicePair";

describe("shownTreeLabels", () => {
  test.each([
    ["both", ["a", "b"]],
    ["left", ["a"]],
    ["right", ["b"]],
  ] as const)("names the trees shown with %s", (trees, labels) => {
    expect(shownTreeLabels(AUSPICE_LABELS, trees)).toStrictEqual(labels);
  });
});

describe("leafNames", () => {
  test("lists the leaves of the shown trees once, sorted", () => {
    expect([leafNames(AUSPICE_PAIR, "both"), leafNames(AUSPICE_PAIR, "left")]).toStrictEqual([
      ["A", "B", "C", "D", "X"],
      ["A", "B", "C", "D", "X"],
    ]);
  });
});
