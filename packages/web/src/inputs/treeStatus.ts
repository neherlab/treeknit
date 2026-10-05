import type { TreeInspection, ValidationError } from "@neherlab/treeknit-wasm";

import { characterRange, type TextRange } from "../ui/codeLines";

export type TreeStatusKind = "pending" | "ok" | "warning" | "error";

export interface StatusMessage {
  severity: "error" | "warning";
  text: string;
}

export interface TreeStatus {
  kind: TreeStatusKind;
  messages: readonly StatusMessage[];
  errorRange: TextRange | undefined;
}

export const TREE_STATUS_LABELS: Record<TreeStatusKind, string> = {
  pending: "Checking the tree",
  ok: "The tree is ready",
  warning: "The tree has warnings",
  error: "The tree has errors",
};

export function treeStatus(
  inspection: TreeInspection | undefined,
  newickErrors: readonly ValidationError[],
): TreeStatus {
  const warnings = (inspection?.warnings ?? []).map((text): StatusMessage => ({ severity: "warning", text }));

  if (inspection?.error !== null && inspection?.error !== undefined) {
    const { message, line, column } = inspection.error;

    return {
      kind: "error",
      messages: [{ severity: "error", text: message }, ...warnings],
      errorRange: line === null || column === null ? undefined : characterRange({ line, column }),
    };
  }

  if (newickErrors.length > 0) {
    return {
      kind: "error",
      messages: [
        ...newickErrors.map(({ message }): StatusMessage => ({ severity: "error", text: message })),
        ...warnings,
      ],
      errorRange: undefined,
    };
  }

  if (inspection === undefined) {
    return { kind: "pending", messages: [], errorRange: undefined };
  }

  return { kind: warnings.length > 0 ? "warning" : "ok", messages: warnings, errorRange: undefined };
}
