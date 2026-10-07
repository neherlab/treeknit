import type { Rgba } from "@neherlab/treeknit-wasm";

const OPAQUE = 255;

export function cssColor([r, g, b, a]: Rgba): string {
  return `rgb(${String(r)} ${String(g)} ${String(b)} / ${String(Math.round((a / OPAQUE) * 1000) / 1000)})`;
}

export function withOpacity([r, g, b, a]: Rgba, opacity: number): Rgba {
  return [r, g, b, Math.round(a * clamp(opacity, 0, 1))];
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(Math.max(value, low), high);
}
