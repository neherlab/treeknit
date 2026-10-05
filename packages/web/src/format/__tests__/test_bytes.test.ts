import { describe, expect, test } from "vitest";

import { formatBytes } from "../bytes";

describe("formatBytes", () => {
  test("writes sizes below one kilobyte in bytes", () => {
    expect([0, 1, 999].map(formatBytes)).toStrictEqual(["0 bytes", "1 byte", "999 bytes"]);
  });

  test("writes sizes below one megabyte in kilobytes with one decimal", () => {
    expect([1000, 1540, 999_949].map(formatBytes)).toStrictEqual(["1 kB", "1.5 kB", "999.9 kB"]);
  });

  test("writes sizes below one gigabyte in megabytes with one decimal", () => {
    expect([1_000_000, 12_345_678, 999_949_999].map(formatBytes)).toStrictEqual(["1 MB", "12.3 MB", "999.9 MB"]);
  });

  test("writes larger sizes in gigabytes with one decimal", () => {
    expect([999_950_000, 2_000_000_000, 12_345_678_901].map(formatBytes)).toStrictEqual(["1 GB", "2 GB", "12.3 GB"]);
  });

  test("moves to the larger unit where rounding would reach 1,000", () => {
    expect([999.4, 999.5, 999_949, 999_950, 999_999].map(formatBytes)).toStrictEqual([
      "999 bytes",
      "1 kB",
      "999.9 kB",
      "1 MB",
      "1 MB",
    ]);
  });

  test("writes a negative or non-finite size as zero bytes", () => {
    expect([-1500, Number.NaN, Number.POSITIVE_INFINITY].map(formatBytes)).toStrictEqual([
      "0 bytes",
      "0 bytes",
      "0 bytes",
    ]);
  });
});
