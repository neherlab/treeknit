import type { BranchLengths, TreeInspection } from "@neherlab/treeknit-wasm";

import { counted } from "../format/count";
import { NONE } from "../format/words";

export const BRANCH_LENGTH_LABELS: Record<BranchLengths, string> = {
  all: "Branch lengths",
  some: "Some branch lengths",
  none: "No branch lengths",
};

export const BRANCH_LENGTH_VALUES: Record<BranchLengths, string> = {
  all: "All",
  some: "Some",
  none: NONE,
};

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
