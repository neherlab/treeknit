import { describe, expect, test } from "vitest";

import { counted, formatCount } from "../count";

describe("formatCount", () => {
  test("groups thousands with commas in English", () => {
    expect([0, 7, 1234, 1_234_567].map(formatCount)).toStrictEqual(["0", "7", "1,234", "1,234,567"]);
  });

  test("rejects a count that is not a finite number instead of writing NaN or ∞", () => {
    expect(() => formatCount(Number.NaN)).toThrow(RangeError);
    expect(() => formatCount(Number.POSITIVE_INFINITY)).toThrow(RangeError);
  });
});

describe("counted", () => {
  test("uses the singular for one and the plural otherwise", () => {
    expect([counted(1, "leaf", "leaves"), counted(0, "leaf", "leaves"), counted(1500, "tree", "trees")]).toStrictEqual([
      "1 leaf",
      "0 leaves",
      "1,500 trees",
    ]);
  });
});
