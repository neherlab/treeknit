import { describe, expect, test } from "vitest";

import { formatBranchLength, leafCount, mccSummary, mccTitle } from "../format";

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

  test("numbers MCCs from 1 and counts their leaves", () => {
    expect([mccTitle(0), leafCount(1), mccSummary(2, 1500)]).toStrictEqual(["MCC 1", "1 leaf", "MCC 3, 1,500 leaves"]);
  });
});
