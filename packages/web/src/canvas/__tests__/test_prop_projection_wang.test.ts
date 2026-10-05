import type { Bezier, Point } from "@neherlab/treeknit-wasm";
import * as fc from "fast-check";
import { describe, expect, test } from "vitest";

import {
  type Column,
  CURVE_SEGMENTS_MAX,
  CURVE_TOLERANCE_PX,
  columnPixel,
  cubicPoint,
  wangSegmentCount,
} from "../projection";

const PROBES_PER_SEGMENT = 16;

describe("wangSegmentCount", () => {
  test("keeps uniform parameter samples within the pixel tolerance of the curve below the cap, as Wang's bound guarantees", () => {
    fc.assert(
      fc.property(genCurve(), genColumn(), fc.double({ min: 0.01, max: 64, noNaN: true }), (curve, column, rowPx) => {
        const segments = wangSegmentCount(curve, column, rowPx);

        expect(segments).toBeLessThanOrEqual(CURVE_SEGMENTS_MAX);

        if (segments < CURVE_SEGMENTS_MAX) {
          expect(largestDeviationPx(curve, column, rowPx, segments) - CURVE_TOLERANCE_PX).toBeLessThan(1e-13);
        }
      }),
    );
  });
});

function largestDeviationPx(curve: Bezier, column: Column, rowPx: number, segments: number): number {
  const pixel = (point: Point): [number, number] => [columnPixel(column, point[0]), point[1] * rowPx];

  const deviations = Array.from({ length: segments * PROBES_PER_SEGMENT }, (_, probe) => {
    const segment = Math.floor(probe / PROBES_PER_SEGMENT);
    const s = (probe % PROBES_PER_SEGMENT) / PROBES_PER_SEGMENT;
    const [ax, ay] = pixel(cubicPoint(curve, segment / segments));
    const [bx, by] = pixel(cubicPoint(curve, (segment + 1) / segments));
    const [cx, cy] = pixel(cubicPoint(curve, (segment + s) / segments));

    return Math.hypot(cx - (ax + s * (bx - ax)), cy - (ay + s * (by - ay)));
  });

  return Math.max(...deviations);
}

function genCurve(): fc.Arbitrary<Bezier> {
  return fc.record({ from: genPoint(), c1: genPoint(), c2: genPoint(), to: genPoint() });
}

function genPoint(): fc.Arbitrary<Point> {
  return fc.tuple(fc.double({ min: 0, max: 1, noNaN: true }), fc.double({ min: 0, max: 20_000, noNaN: true }));
}

function genColumn(): fc.Arbitrary<Column> {
  return fc
    .record({
      start: fc.integer({ min: 0, max: 800 }),
      width: fc.integer({ min: 20, max: 600 }),
      mirrored: fc.boolean(),
    })
    .map(({ start, width, mirrored }) => ({ start, end: start + width, mirrored }));
}
