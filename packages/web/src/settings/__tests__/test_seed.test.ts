import * as fc from "fast-check";
import { describe, expect, test } from "vitest";

import { randomSeed } from "../seed";

const LARGEST_DRAW = 2 ** 53 - 1;

function draws(...values: number[]): () => number {
  const remaining = values.values();

  return () => {
    const next = remaining.next();

    if (next.done === true) {
      throw new Error("No draws left");
    }

    return next.value;
  };
}

describe("new seed", () => {
  test("takes the draw as is when the largest seed spans every draw", () => {
    expect(randomSeed(draws(LARGEST_DRAW), LARGEST_DRAW)).toBe(LARGEST_DRAW);
  });

  test("maps a draw into the seed range by its remainder", () => {
    expect(randomSeed(draws(70_000), 65_535)).toBe(70_000 - 65_536);
  });

  test("draws again when the draw falls into the incomplete last block, so every seed is equally likely", () => {
    expect(randomSeed(draws(LARGEST_DRAW, LARGEST_DRAW - 1, 7), 2)).toBe(1);
  });

  test("keeps a draw below the last complete block", () => {
    expect(randomSeed(draws(LARGEST_DRAW - 2), 2)).toBe(2);
  });

  test("uses the 32-bit range without an upper bound", () => {
    expect(randomSeed(draws(2 ** 32 + 12), null)).toBe(12);
  });

  test("returns a whole number from zero to the largest seed for any draw", () => {
    fc.assert(
      fc.property(fc.maxSafeNat(), fc.maxSafeNat(), (draw, max) => {
        const seed = randomSeed(draws(draw, 0), max);

        expect(Number.isSafeInteger(seed) && seed >= 0 && seed <= max).toBe(true);
      }),
    );
  });
});
