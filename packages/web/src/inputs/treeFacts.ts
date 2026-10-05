import type { TreeInspection } from "@neherlab/treeknit-wasm";

import { BRANCH_LENGTH_LABELS } from "./treeStatus";

const COUNT = new Intl.NumberFormat("en");

const GRID_NAVIGATION_KEYS: ReadonlySet<string> = new Set([
  "ArrowUp",
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  " ",
]);

export function treeFacts(inspection: TreeInspection | undefined, missing: number | undefined): string[] {
  if (inspection === undefined || inspection.error !== null) {
    return [];
  }

  return [
    counted(inspection.leaves, "leaf", "leaves"),
    ...(missing === undefined || missing === 0 ? [] : [`${counted(missing, "leaf", "leaves")} missing`]),
    ...(inspection.polytomies === 0 ? [] : [counted(inspection.polytomies, "polytomy", "polytomies")]),
    BRANCH_LENGTH_LABELS[inspection.branchLengths],
  ];
}

export function counted(count: number, one: string, many: string): string {
  return `${COUNT.format(count)} ${count === 1 ? one : many}`;
}

export function isGridNavigationKey(key: string, withModifier: boolean): boolean {
  return !withModifier && GRID_NAVIGATION_KEYS.has(key);
}
