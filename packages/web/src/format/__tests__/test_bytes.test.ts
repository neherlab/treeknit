import { describe, expect, test } from "vitest";

import { formatBytes } from "../bytes";

describe("formatBytes", () => {
  test("writes sizes below one kilobyte in bytes", () => {
    expect([0, 1, 999].map(formatBytes)).toStrictEqual(["0 bytes", "1 byte", "999 bytes"]);
  });

  test("writes sizes below one megabyte in kilobytes with one decimal", () => {
    expect([1000, 1540, 999_949].map(formatBytes)).toStrictEqual(["1 kB", "1.5 kB", "999.9 kB"]);
  });

  test("writes larger sizes in megabytes with one decimal", () => {
    expect([1_000_000, 12_345_678, 2_000_000_000].map(formatBytes)).toStrictEqual(["1 MB", "12.3 MB", "2,000 MB"]);
  });
});
