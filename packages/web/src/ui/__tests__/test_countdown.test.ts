import { describe, expect, test } from "vitest";

import { remainingSeconds } from "../countdown";

describe("remainingSeconds", () => {
  test.each([
    [0, 10],
    [1, 10],
    [999, 10],
    [1000, 9],
    [9001, 1],
    [10_000, 0],
    [12_000, 0],
  ])("after %i ms of a 10 s timeout shows %i s", (elapsedMs, seconds) => {
    expect(remainingSeconds(10_000, elapsedMs)).toBe(seconds);
  });
});
