import { describe, expect, test } from "vitest";

import { FADE_IN_MS, fadeDone, fadeInOpacity, reducedFade, shownOpacity } from "../motion";

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

describe("shownOpacity", () => {
  const FADING = { key: "result-1", opacity: 0.4 };

  test("shows the fade of the result it belongs to", () => {
    expect(shownOpacity(FADING, "result-1", false)).toBe(0.4);
  });

  test("starts a new result transparent before its first animation frame", () => {
    expect(shownOpacity(FADING, "result-2", false)).toBe(0);
  });

  test("shows every result opaque under reduced motion", () => {
    expect(shownOpacity(FADING, "result-2", true)).toBe(1);
  });
});

describe("fadeDone", () => {
  test("is done only when the fade of the shown result is opaque", () => {
    expect(fadeDone({ key: "result-1", opacity: 1 }, "result-1")).toBe(true);
    expect(fadeDone({ key: "result-1", opacity: 0.4 }, "result-1")).toBe(false);
    expect(fadeDone({ key: "result-1", opacity: 1 }, "result-2")).toBe(false);
  });
});

describe("reducedFade", () => {
  test("ends a running fade at full opacity when reduced motion turns on, so turning it off again starts no fade", () => {
    const settled = reducedFade({ key: "result-1", opacity: 0.4 }, "result-1", true);

    expect(settled).toStrictEqual({ key: "result-1", opacity: 1 });
    expect(settled !== undefined && fadeDone(settled, "result-1")).toBe(true);
  });

  test("marks a new result as shown under reduced motion", () => {
    expect(reducedFade({ key: "result-1", opacity: 1 }, "result-2", true)).toStrictEqual({
      key: "result-2",
      opacity: 1,
    });
  });

  test("changes nothing without reduced motion or after the fade", () => {
    expect(reducedFade({ key: "result-1", opacity: 0.4 }, "result-1", false)).toBeUndefined();
    expect(reducedFade({ key: "result-1", opacity: 1 }, "result-1", true)).toBeUndefined();
  });
});
