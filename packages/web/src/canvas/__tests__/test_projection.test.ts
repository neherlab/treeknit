import { describe, expect, test } from "vitest";

import type { Bezier, Point } from "@neherlab/treeknit-wasm";

import {
  type Column,
  columnPixel,
  cubicPoint,
  projectPath,
  sampleCubic,
  sampleCubicChain,
} from "../projection";

const LEFT: Column = { start: 10, end: 210, mirrored: false };

const RIGHT: Column = { start: 600, end: 800, mirrored: true };

const LINK: Bezier = { from: [0, 2], c1: [0.5, 2], c2: [0.5, 7], to: [1, 7] };

const BACK: Bezier = { from: [1, 7], c1: [0.5, 7], c2: [0.5, 2], to: [0, 2] };

describe("columnPixel", () => {
  test("maps 0 and 1 to the column edges, root side first", () => {
    expect([columnPixel(LEFT, 0), columnPixel(LEFT, 0.25), columnPixel(LEFT, 1)]).toStrictEqual([10, 60, 210]);
  });

  test("mirrors a column so its root sits at the right edge", () => {
    expect([columnPixel(RIGHT, 0), columnPixel(RIGHT, 0.25), columnPixel(RIGHT, 1)]).toStrictEqual([800, 750, 600]);
  });
});

describe("projectPath", () => {
  const elbow: Point[] = [
    [0, 0.5],
    [0, 1],
    [0.5, 1],
  ];

  test("puts pixels on x and rows on y on the wide layout", () => {
    expect(projectPath(elbow, LEFT, "y")).toStrictEqual([
      [10, 0.5],
      [10, 1],
      [110, 1],
    ]);
  });

  test("puts rows on x and pixels on y on the narrow layout", () => {
    expect(projectPath(elbow, LEFT, "x")).toStrictEqual([
      [0.5, 10],
      [1, 10],
      [1, 110],
    ]);
  });
});

describe("sampleCubic", () => {
  test("starts and ends exactly at the curve endpoints", () => {
    const points = sampleCubic(LINK, 16);

    expect(points).toHaveLength(17);
    expect(points[0]).toStrictEqual(LINK.from);
    expect(points.at(-1)).toStrictEqual(LINK.to);
  });

  test("passes through the midpoint of the S-curve at x = 0.5", () => {
    expect(sampleCubic(LINK, 2)[1]).toStrictEqual([0.5, 4.5]);
  });

  test("keeps every sample between the endpoints of a monotone S-curve, in increasing x", () => {
    const points = sampleCubic(LINK, 32);
    const xs = points.map(([x]) => x);
    const ys = points.map(([, y]) => y);

    expect(xs).toStrictEqual(xs.toSorted((left, right) => left - right));
    expect(Math.min(...ys)).toBe(2);
    expect(Math.max(...ys)).toBe(7);
  });

  test("evaluates the Bernstein form at quarter points", () => {
    expect(cubicPoint(LINK, 0.25)).toStrictEqual([0.296875, 2 + 5 * 0.15625]);
  });
});

describe("sampleCubicChain", () => {
  test("joins segments without repeating the shared point", () => {
    const points = sampleCubicChain([LINK, BACK], 4);

    expect(points).toHaveLength(9);
    expect(points[4]).toStrictEqual(LINK.to);
    expect(points[0]).toStrictEqual(points.at(-1));
  });

  test("returns no point for no segment", () => {
    expect(sampleCubicChain([])).toStrictEqual([]);
  });
});
