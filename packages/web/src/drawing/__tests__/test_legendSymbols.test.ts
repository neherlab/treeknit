import { describe, expect, test } from "vitest";

import { bandPath, horizontalPath, sCurvePath } from "../legendSymbols";

describe("legend symbol paths", () => {
  test("draws a line across the middle of a 12 px high symbol to its width", () => {
    expect([horizontalPath(24), horizontalPath(16)]).toStrictEqual(["M0 6 H24", "M0 6 H16"]);
  });

  test("draws an S-curve with its control points halfway, or at a given x", () => {
    expect([sCurvePath(2, 10, 24), sCurvePath(1, 6, 16, 9)]).toStrictEqual([
      "M0 2 C12 2 12 10 24 10",
      "M0 1 C9 1 9 6 16 6",
    ]);
  });

  test("closes a band between two S-curves a thickness apart", () => {
    expect(bandPath(1, 5, 6, 24)).toBe("M0 1 C12 1 12 5 24 5 V11 C12 11 12 7 0 7 Z");
  });
});
