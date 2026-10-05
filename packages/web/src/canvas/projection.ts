import { type LeafAxis, worldPosition } from "./viewState";

export interface NormalizedPoint {
  x: number;
  y: number;
}

export type CubicBezier = readonly [NormalizedPoint, NormalizedPoint, NormalizedPoint, NormalizedPoint];

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

export function projectPoint(point: NormalizedPoint, column: Column, leafAxis: LeafAxis): WorldPosition {
  return worldPosition(leafAxis, columnPixel(column, point.x), point.y);
}

export function projectPath(points: readonly NormalizedPoint[], column: Column, leafAxis: LeafAxis): WorldPosition[] {
  return points.map((point) => projectPoint(point, column, leafAxis));
}

export function cubicPoint([p0, p1, p2, p3]: CubicBezier, t: number): NormalizedPoint {
  const u = 1 - t;
  const w0 = u * u * u;
  const w1 = 3 * u * u * t;
  const w2 = 3 * u * t * t;
  const w3 = t * t * t;

  return {
    x: w0 * p0.x + w1 * p1.x + w2 * p2.x + w3 * p3.x,
    y: w0 * p0.y + w1 * p1.y + w2 * p2.y + w3 * p3.y,
  };
}

export function sampleCubic(curve: CubicBezier, samples = CURVE_SAMPLES): NormalizedPoint[] {
  const count = Math.max(1, Math.round(samples));
  const points = Array.from({ length: count - 1 }, (_, index) => cubicPoint(curve, (index + 1) / count));

  return [curve[0], ...points, curve[3]];
}

export function sampleCubicChain(curves: readonly CubicBezier[], samples = CURVE_SAMPLES): NormalizedPoint[] {
  return curves.flatMap((curve, index) => {
    const points = sampleCubic(curve, samples);

    return index === 0 ? points : points.slice(1);
  });
}
