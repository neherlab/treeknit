import type { Bezier, Point } from "@neherlab/treeknit-wasm";
import * as fc from "fast-check";
import { describe, expect, test } from "vitest";

import { type Column, CURVE_SEGMENTS_MAX, CURVE_TOLERANCE_PX, cubicPoint, wangSegmentCount } from "../projection";

const PROBES_PER_SEGMENT = 16;

describe("wangSegmentCount", () => {
  test("keeps uniform parameter samples within the pixel tolerance of the curve, as Wang's bound guarantees", () => {
    fc.assert(
      fc.property(genCurve(), genColumn(), fc.double({ min: 1, max: 64, noNaN: true }), (curve, column, rowPx) => {
        const segments = wangSegmentCount(curve, column, rowPx);

        fc.pre(segments < CURVE_SEGMENTS_MAX);

        const excess = largestDeviationPx(curve, column, rowPx, segments) - CURVE_TOLERANCE_PX;

        expect(excess).toBeLessThan(1e-13);
      }),
    );
  });
});

function largestDeviationPx(curve: Bezier, column: Column, rowPx: number, segments: number): number {
  const width = column.end - column.start;

  const deviations = Array.from({ length: segments * PROBES_PER_SEGMENT }, (_, probe) => {
    const segment = Math.floor(probe / PROBES_PER_SEGMENT);
    const s = (probe % PROBES_PER_SEGMENT) / PROBES_PER_SEGMENT;
    const [ax, ay] = cubicPoint(curve, segment / segments);
    const [bx, by] = cubicPoint(curve, (segment + 1) / segments);
    const [cx, cy] = cubicPoint(curve, (segment + s) / segments);

    return Math.hypot((cx - (ax + s * (bx - ax))) * width, (cy - (ay + s * (by - ay))) * rowPx);
  });

  return Math.max(...deviations);
}

function genCurve(): fc.Arbitrary<Bezier> {
  return fc.record({ from: genPoint(), c1: genPoint(), c2: genPoint(), to: genPoint() });
}

function genPoint(): fc.Arbitrary<Point> {
  return fc.tuple(fc.double({ min: 0, max: 1, noNaN: true }), fc.double({ min: 0, max: 40, noNaN: true }));
}

function genColumn(): fc.Arbitrary<Column> {
  return fc.integer({ min: 20, max: 600 }).map((width) => ({ start: 0, end: width, mirrored: false }));
}
