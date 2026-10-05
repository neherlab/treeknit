import type { DrawingRules } from "@neherlab/treeknit-wasm";

export function innerWidthPx(crossPx: number, rules: DrawingRules): number {
  return Math.max(crossPx - 2 * rules.marginPx, 0);
}

export function labelColumnPx(maxPx: number, longestPx: number, rules: DrawingRules): number {
  return longestPx <= 0 ? 0 : Math.min(longestPx + 2 * rules.labelGapPx, maxPx);
}
