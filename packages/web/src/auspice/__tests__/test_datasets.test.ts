import { describe, expect, test } from "vitest";

import { leafNames } from "../datasets";
import { AUSPICE_PAIR } from "./auspicePair";

describe("leafNames", () => {
  test("lists the leaves of the shown trees once, sorted", () => {
    expect([leafNames(AUSPICE_PAIR, "both"), leafNames(AUSPICE_PAIR, "left")]).toStrictEqual([
      ["A", "B", "C", "D", "X"],
      ["A", "B", "C", "D", "X"],
    ]);
  });
});
