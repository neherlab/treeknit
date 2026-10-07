import { describe, expect, test } from "vitest";

import { formatBranchLength, leafCount, mccSummary } from "../format";

describe("drawing text", () => {
  test("writes a branch length with at most four significant digits and English grouping", () => {
    expect([0, 0.1, 0.0123456, 1234.56, 98_765.4].map(formatBranchLength)).toStrictEqual([
      "0",
      "0.1",
      "0.01235",
      "1,235",
      "98,770",
    ]);
  });

  test("writes a missing branch length as None", () => {
    expect(formatBranchLength(null)).toBe("None");
  });

  test("counts leaves and joins the count to the MCC name", () => {
    expect([leafCount(1), mccSummary("MCC 3", 1500)]).toStrictEqual(["1 leaf", "MCC 3, 1,500 leaves"]);
  });
});
