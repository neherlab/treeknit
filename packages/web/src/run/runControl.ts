import type { Progress, ValidationError } from "@neherlab/treeknit-wasm";
import { match } from "ts-pattern";

export const ADD_ANOTHER_TREE = "Add at least one more tree";

export const FIX_THE_ERRORS = "Fix the errors above";

export const CHECKING_INPUT = "Checking the trees and settings";

export const MATCHING_TOPOLOGIES = "Matching topologies";

export const STARTING_RUN = "Starting";

const MIN_TREES = 2;

export interface RunReadiness {
  treeCount: number;
  hasDraft: boolean;
  checking: boolean;
  errors: readonly ValidationError[];
}

export function runBlockedReason({ treeCount, hasDraft, checking, errors }: RunReadiness): string | null {
  if (treeCount < MIN_TREES) {
    return ADD_ANOTHER_TREE;
  }

  if (hasDraft) {
    return FIX_THE_ERRORS;
  }

  if (checking) {
    return CHECKING_INPUT;
  }

  return errors.length > 0 ? FIX_THE_ERRORS : null;
}

export function visibleGeneralErrors(
  treeCount: number,
  general: readonly ValidationError[],
): readonly ValidationError[] {
  return treeCount < MIN_TREES ? [] : general;
}

export function progressLabel(progress: Progress | null): string {
  if (progress === null) {
    return STARTING_RUN;
  }

  return match(progress.phase)
    .with("matching", () => MATCHING_TOPOLOGIES)
    .with("pairs", "done", () => roundAndPair(progress))
    .exhaustive();
}

export function progressPercent(progress: Progress | null): number | null {
  if (progress === null || progress.phase === "matching") {
    return null;
  }

  return Math.round(progress.fraction * 100);
}

function roundAndPair({ round, rounds, pair, pairs }: Progress): string {
  return `Round ${String(round)} of ${String(rounds)}, pair ${String(pair)} of ${String(pairs)}`;
}

export function isRunShortcut(key: string, ctrlKey: boolean, metaKey: boolean): boolean {
  return key === "Enter" && (ctrlKey || metaKey);
}
