import type { TreeInspection } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { counted } from "../../format/count";
import { BRANCH_LENGTH_VALUES, treeFacts } from "../treeFacts";

const INSPECTION: TreeInspection = {
  label: "ha",
  leaves: 1234,
  internalNodes: 1200,
  polytomies: 3,
  branchLengths: "some",
  warnings: [],
  error: null,
};

describe("tree facts", () => {
  test("lists leaves, missing leaves, polytomies, and branch lengths", () => {
    expect(treeFacts(INSPECTION, 12)).toStrictEqual([
      "1,234 leaves",
      "12 leaves missing",
      "3 polytomies",
      "Some branch lengths",
    ]);
  });

  test("leaves out zero counts and an overlap that is not known yet", () => {
    expect(treeFacts({ ...INSPECTION, leaves: 1, polytomies: 0, branchLengths: "all" }, undefined)).toStrictEqual([
      "1 leaf",
      "Branch lengths",
    ]);
  });

  test("has no facts for a tree that does not parse", () => {
    expect(treeFacts({ ...INSPECTION, error: { message: "expected ';'", line: 1, column: 9 } }, 0)).toStrictEqual([]);
  });

  test("counts in singular and plural", () => {
    expect([counted(1, "polytomy", "polytomies"), counted(2, "polytomy", "polytomies")]).toStrictEqual([
      "1 polytomy",
      "2 polytomies",
    ]);
  });

  test("names how many branches of a tree have lengths in the overview table", () => {
    expect(BRANCH_LENGTH_VALUES).toStrictEqual({ all: "All", some: "Some", none: "None" });
  });
});
