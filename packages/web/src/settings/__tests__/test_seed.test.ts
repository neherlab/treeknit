import { describe, expect, test } from "vitest";

import { randomSeed } from "../seed";

describe("new seed", () => {
  test("takes the random 32-bit value", () => {
    expect(randomSeed(Uint32Array.of(4_294_967_295), 9_007_199_254_740_991)).toBe(4_294_967_295);
  });

  test("stays within the largest seed of the schema", () => {
    expect(randomSeed(Uint32Array.of(70_000), 65_535)).toBe(65_535);
  });

  test("uses the random value as is without an upper bound", () => {
    expect(randomSeed(Uint32Array.of(12), null)).toBe(12);
  });
});
