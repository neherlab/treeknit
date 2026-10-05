import { describe, expect, test } from "vitest";

import { formatElapsed } from "../elapsed";

describe("formatElapsed", () => {
  test("writes minutes and two-digit seconds, rounding down", () => {
    expect([0, 999, 1000, 65_432, 600_000].map(formatElapsed)).toStrictEqual(["0:00", "0:00", "0:01", "1:05", "10:00"]);
  });

  test("keeps counting minutes past one hour", () => {
    expect(formatElapsed(3_725_000)).toBe("62:05");
  });

  test("writes a negative or non-finite span as zero", () => {
    expect([-5000, Number.NaN, Number.POSITIVE_INFINITY].map(formatElapsed)).toStrictEqual(["0:00", "0:00", "0:00"]);
  });
});
