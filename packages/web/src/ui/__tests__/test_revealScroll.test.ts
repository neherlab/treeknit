import { describe, expect, test } from "vitest";

import { revealScroll } from "../revealScroll";

const VIEW = { start: 100, end: 300 };

describe("revealScroll", () => {
  test("does not scroll for an item inside the view", () => {
    expect(revealScroll({ start: 120, end: 140 }, VIEW)).toBe(0);
  });

  test("scrolls back so that an item above the view starts at its start", () => {
    expect(revealScroll({ start: 40, end: 60 }, VIEW)).toBe(-60);
  });

  test("scrolls back for an item that starts above the view and ends inside it", () => {
    expect(revealScroll({ start: 90, end: 110 }, VIEW)).toBe(-10);
  });

  test("scrolls forward so that an item below the view ends at its end", () => {
    expect(revealScroll({ start: 400, end: 420 }, VIEW)).toBe(120);
  });

  test("scrolls forward only until the start of an item longer than the view", () => {
    expect(revealScroll({ start: 150, end: 600 }, VIEW)).toBe(50);
  });
});
