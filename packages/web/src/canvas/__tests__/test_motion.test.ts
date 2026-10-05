import { describe, expect, test } from "vitest";

import { FADE_IN_MS, fadeInOpacity } from "../motion";

describe("fadeInOpacity", () => {
  test("starts transparent and ends opaque after the fade duration", () => {
    expect(fadeInOpacity(0)).toBe(0);
    expect(fadeInOpacity(FADE_IN_MS)).toBe(1);
    expect(fadeInOpacity(FADE_IN_MS * 3)).toBe(1);
  });

  test("rises monotonically with an ease-out curve", () => {
    const steps = Array.from({ length: 11 }, (_, index) => fadeInOpacity((index * FADE_IN_MS) / 10));

    expect(steps).toStrictEqual(steps.toSorted((left, right) => left - right));
    expect(fadeInOpacity(FADE_IN_MS / 2)).toBe(0.75);
  });

  test("treats a time before the start as transparent and a zero duration as opaque", () => {
    expect(fadeInOpacity(-50)).toBe(0);
    expect(fadeInOpacity(10, 0)).toBe(1);
  });
});
