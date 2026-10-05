import type { Bezier, Point } from "@neherlab/treeknit-wasm";

import { type LeafAxis, worldPosition } from "./viewState";

export interface Column {
  start: number;
  end: number;
  mirrored: boolean;
}

export type WorldPosition = [number, number];

export const CURVE_SAMPLES = 16;

export function columnPixel({ start, end, mirrored }: Column, x: number): number {
  return mirrored ? end - x * (end - start) : start + x * (end - start);
}

export function projectPoint([x, y]: Point, column: Column, leafAxis: LeafAxis): WorldPosition {
  return worldPosition(leafAxis, columnPixel(column, x), y);
}

export function projectPath(points: readonly Point[], column: Column, leafAxis: LeafAxis): WorldPosition[] {
  return points.map((point) => projectPoint(point, column, leafAxis));
}

export function cubicPoint({ from, c1, c2, to }: Bezier, t: number): Point {
  const u = 1 - t;
  const w0 = u * u * u;
  const w1 = 3 * u * u * t;
  const w2 = 3 * u * t * t;
  const w3 = t * t * t;

  return [
    w0 * from[0] + w1 * c1[0] + w2 * c2[0] + w3 * to[0],
    w0 * from[1] + w1 * c1[1] + w2 * c2[1] + w3 * to[1],
  ];
}

export function sampleCubic(curve: Bezier, samples = CURVE_SAMPLES): Point[] {
  const count = Math.max(1, Math.round(samples));
  const points = Array.from({ length: count - 1 }, (_, index) => cubicPoint(curve, (index + 1) / count));

  return [curve.from, ...points, curve.to];
}

export function sampleCubicChain(curves: readonly Bezier[], samples = CURVE_SAMPLES): Point[] {
  return curves.flatMap((curve, index) => {
    const points = sampleCubic(curve, samples);

    return index === 0 ? points : points.slice(1);
  });
}
