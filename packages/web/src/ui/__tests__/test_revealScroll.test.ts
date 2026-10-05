import { describe, expect, test } from "vitest";

import { revealOffset, revealScroll } from "../revealScroll";

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

describe("revealOffset", () => {
  const SCROLLER = {
    offsetWidth: 402,
    offsetHeight: 202,
    clientTop: 1,
    clientLeft: 1,
    clientWidth: 400,
    clientHeight: 200,
  };

  test("scrolls a mark below the scroller up to its bottom edge, in layout pixels", () => {
    const view = { top: 50, left: 10, width: 402, height: 202 };
    const mark = { top: 291, left: 31, width: 8, height: 20 };

    expect(revealOffset(mark, view, SCROLLER)).toStrictEqual({ top: 60, left: 0 });
  });

  test("divides screen distances by the scale of a CSS transform", () => {
    const view = { top: 50, left: 10, width: 804, height: 404 };
    const mark = { top: 532, left: 32, width: 16, height: 40 };

    expect(revealOffset(mark, view, SCROLLER)).toStrictEqual({ top: 60, left: 0 });
  });

  test("scrolls left to a mark before the visible columns", () => {
    const view = { top: 0, left: 100, width: 402, height: 202 };
    const mark = { top: 21, left: 61, width: 8, height: 20 };

    expect(revealOffset(mark, view, SCROLLER)).toStrictEqual({ top: 0, left: -40 });
  });
});
