import type { BranchLengths, TreeInspection, ValidationError } from "@neherlab/treeknit-wasm";

import { characterRange, type TextRange } from "../ui/codeLines";

export type TreeStatusKind = "pending" | "ok" | "warning" | "error";

export interface TreeStatus {
  kind: TreeStatusKind;
  messages: readonly string[];
  errorRange: TextRange | undefined;
}

export const TREE_STATUS_LABELS: Record<TreeStatusKind, string> = {
  pending: "Checking the tree",
  ok: "The tree is ready",
  warning: "The tree has warnings",
  error: "The tree has errors",
};

export const BRANCH_LENGTH_LABELS: Record<BranchLengths, string> = {
  all: "Branch lengths",
  some: "Some branch lengths",
  none: "No branch lengths",
};

export function treeStatus(
  inspection: TreeInspection | undefined,
  newickErrors: readonly ValidationError[],
): TreeStatus {
  if (inspection?.error !== null && inspection?.error !== undefined) {
    const { message, line, column } = inspection.error;

    return {
      kind: "error",
      messages: [message],
      errorRange: line === null || column === null ? undefined : characterRange({ line, column }),
    };
  }

  if (newickErrors.length > 0) {
    return { kind: "error", messages: newickErrors.map(({ message }) => message), errorRange: undefined };
  }

  if (inspection === undefined) {
    return { kind: "pending", messages: [], errorRange: undefined };
  }

  return {
    kind: inspection.warnings.length > 0 ? "warning" : "ok",
    messages: inspection.warnings,
    errorRange: undefined,
  };
}
