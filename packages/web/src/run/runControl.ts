import type { Progress, ValidationError } from "@neherlab/treeknit-wasm";
import { match } from "ts-pattern";

import { formatSpan } from "../format/elapsed";

export const ADD_ANOTHER_TREE = "Add at least one more tree";

export const FIX_THE_ERRORS = "Fix the errors above";

export const CHECKING_INPUT = "Checking the trees and settings";

export const CHECK_FAILED = "The trees and settings could not be checked";

export const MATCHING_TOPOLOGIES = "Matching topologies";

export const STARTING_RUN = "Starting";

const MIN_TREES = 2;

const SECOND_MS = 1000;

export type ValidationState = "checking" | "failed" | "checked";

export interface ValidationQueryState {
  status: "pending" | "error" | "success";
  isPlaceholderData: boolean;
}

export interface RunReadiness {
  treeCount: number;
  hasDraft: boolean;
  validation: ValidationState;
  errors: readonly ValidationError[];
}

export function validationState({ status, isPlaceholderData }: ValidationQueryState): ValidationState {
  return match(status)
    .with("error", () => "failed" as const)
    .with("pending", () => "checking" as const)
    .with("success", () => (isPlaceholderData ? "checking" : "checked"))
    .exhaustive();
}

export function runBlockedReason({ treeCount, hasDraft, validation, errors }: RunReadiness): string | null {
  if (treeCount < MIN_TREES) {
    return ADD_ANOTHER_TREE;
  }

  if (hasDraft) {
    return FIX_THE_ERRORS;
  }

  if (validation === "checking") {
    return CHECKING_INPUT;
  }

  if (validation === "failed") {
    return CHECK_FAILED;
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

export type ShortcutKey = Pick<
  KeyboardEvent,
  "key" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey" | "repeat" | "isComposing"
>;

export function isRunShortcut({ key, ctrlKey, metaKey, altKey, shiftKey, repeat, isComposing }: ShortcutKey): boolean {
  return key === "Enter" && ctrlKey !== metaKey && !altKey && !shiftKey && !repeat && !isComposing;
}

export function isBehindModal(element: Element): boolean {
  return element.closest("[inert]") !== null;
}

export function runTime(durationMs: number): string {
  return durationMs < SECOND_MS ? `under ${formatSpan(SECOND_MS)}` : formatSpan(durationMs);
}
