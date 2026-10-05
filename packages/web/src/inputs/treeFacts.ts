import type { TreeInspection } from "@neherlab/treeknit-wasm";

import { counted } from "../drawing/format";
import { BRANCH_LENGTH_LABELS } from "./treeStatus";

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

export function isGridNavigationKey(key: string, withModifier: boolean): boolean {
  return !withModifier && GRID_NAVIGATION_KEYS.has(key);
}
