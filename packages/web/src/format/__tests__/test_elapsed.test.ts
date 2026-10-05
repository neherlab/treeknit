import { describe, expect, test } from "vitest";

import { formatElapsed, formatSpan } from "../elapsed";

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

describe("formatSpan", () => {
  test("writes the two largest units, rounding seconds down", () => {
    expect([4200, 42_000, 90_000, 3_905_000, 11_520_000].map((ms) => formatSpan(ms))).toStrictEqual([
      "4s",
      "42s",
      "1m 30s",
      "1h 5m",
      "3h 12m",
    ]);
  });

  test("drops a zero second unit and never adds a third", () => {
    expect([120_000, 7_212_000, 3_661_000].map((ms) => formatSpan(ms))).toStrictEqual(["2m", "2h", "1h 1m"]);
  });

  test("stops at weeks", () => {
    expect([864_000_000, 3_888_000_000].map((ms) => formatSpan(ms))).toStrictEqual(["1w 3d", "6w 3d"]);
  });

  test("writes a span under one second, a negative span, and a non-finite span as now", () => {
    expect([0, 999, -5000, Number.NaN].map((ms) => formatSpan(ms))).toStrictEqual(["now", "now", "now", "now"]);
  });
});
