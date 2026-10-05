import { describe, expect, test } from "vitest";

import { labelColumnPx } from "../labelWidth";

describe("labelColumnPx", () => {
  test("fits the longest label with a 6 px gap on both sides", () => {
    expect(labelColumnPx(600, 80)).toBe(92);
  });

  test("takes at most a quarter of the tree's share of the width", () => {
    expect(labelColumnPx(600, 900)).toBe(150);
  });

  test("takes no space without labels", () => {
    expect(labelColumnPx(600, 0)).toBe(0);
  });
});
