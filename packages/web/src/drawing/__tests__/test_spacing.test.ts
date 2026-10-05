import { describe, expect, test } from "vitest";

import { innerWidthPx, labelColumnPx } from "../spacing";
import { exampleDrawingRules } from "./fixtures";

const RULES = exampleDrawingRules();

describe("innerWidthPx", () => {
  test("leaves a 16 px margin on each side", () => {
    expect(innerWidthPx(600, RULES)).toBe(568);
  });

  test("is zero when the margins fill the drawing", () => {
    expect(innerWidthPx(20, RULES)).toBe(0);
  });
});

describe("labelColumnPx", () => {
  test("fits the longest label with a 6 px gap on both sides", () => {
    expect(labelColumnPx(150, 80, RULES)).toBe(92);
  });

  test("takes at most the given width", () => {
    expect(labelColumnPx(150, 900, RULES)).toBe(150);
  });

  test("takes no space without labels", () => {
    expect(labelColumnPx(150, 0, RULES)).toBe(0);
  });
});
