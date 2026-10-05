import type { Progress, ValidationError } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import {
  ADD_ANOTHER_TREE,
  CHECKING_INPUT,
  FIX_THE_ERRORS,
  isRunShortcut,
  MATCHING_TOPOLOGIES,
  progressLabel,
  progressPercent,
  runBlockedReason,
  type ShortcutKey,
  STARTING_RUN,
  visibleGeneralErrors,
} from "../runControl";

const ERROR: ValidationError = { field: null, message: "need at least two trees", line: null, column: null };

const PROGRESS: Progress = { phase: "pairs", fraction: 0.425, round: 1, rounds: 2, pair: 2, pairs: 3 };

describe("run control", () => {
  test("asks for another tree before anything else", () => {
    expect({
      none: runBlockedReason({ treeCount: 0, hasDraft: false, checking: false, errors: [] }),
      one: runBlockedReason({ treeCount: 1, hasDraft: true, checking: false, errors: [ERROR] }),
    }).toStrictEqual({ none: ADD_ANOTHER_TREE, one: ADD_ANOTHER_TREE });
  });

  test("blocks a run on an invalid draft or a validation error", () => {
    expect({
      draft: runBlockedReason({ treeCount: 2, hasDraft: true, checking: false, errors: [] }),
      error: runBlockedReason({ treeCount: 3, hasDraft: false, checking: false, errors: [ERROR] }),
      ready: runBlockedReason({ treeCount: 2, hasDraft: false, checking: false, errors: [] }),
    }).toStrictEqual({ draft: FIX_THE_ERRORS, error: FIX_THE_ERRORS, ready: null });
  });

  test("blocks a run while the current trees and settings are checked, even when the last check found no errors", () => {
    expect({
      checking: runBlockedReason({ treeCount: 2, hasDraft: false, checking: true, errors: [] }),
      draft: runBlockedReason({ treeCount: 2, hasDraft: true, checking: true, errors: [] }),
      oneTree: runBlockedReason({ treeCount: 1, hasDraft: false, checking: true, errors: [] }),
    }).toStrictEqual({ checking: CHECKING_INPUT, draft: FIX_THE_ERRORS, oneTree: ADD_ANOTHER_TREE });
  });

  test("hides the general errors while the reason asks for another tree", () => {
    expect({ one: visibleGeneralErrors(1, [ERROR]), two: visibleGeneralErrors(2, [ERROR]) }).toStrictEqual({
      one: [],
      two: [ERROR],
    });
  });

  test("names the round and pair in progress, or topology matching", () => {
    expect({
      start: progressLabel(null),
      pairs: progressLabel(PROGRESS),
      matching: progressLabel({ ...PROGRESS, phase: "matching" }),
      done: progressLabel({ ...PROGRESS, phase: "done", fraction: 1, round: 2, pair: 3 }),
    }).toStrictEqual({
      start: STARTING_RUN,
      pairs: "Round 1 of 2, pair 2 of 3",
      matching: MATCHING_TOPOLOGIES,
      done: "Round 2 of 2, pair 3 of 3",
    });
  });

  test("shows a percentage, and an indeterminate bar before the first event and while matching", () => {
    expect({
      start: progressPercent(null),
      pairs: progressPercent(PROGRESS),
      matching: progressPercent({ ...PROGRESS, phase: "matching" }),
      done: progressPercent({ ...PROGRESS, phase: "done", fraction: 1 }),
    }).toStrictEqual({ start: null, pairs: 43, matching: null, done: 100 });
  });

  test("runs on exactly Ctrl+Enter or Cmd+Enter", () => {
    const key: ShortcutKey = {
      key: "Enter",
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      shiftKey: false,
      repeat: false,
      isComposing: false,
    };

    expect({
      ctrl: isRunShortcut({ ...key, ctrlKey: true }),
      cmd: isRunShortcut({ ...key, metaKey: true }),
      plain: isRunShortcut(key),
      escape: isRunShortcut({ ...key, key: "Escape", ctrlKey: true }),
      ctrlCmd: isRunShortcut({ ...key, ctrlKey: true, metaKey: true }),
      ctrlShift: isRunShortcut({ ...key, ctrlKey: true, shiftKey: true }),
      ctrlAlt: isRunShortcut({ ...key, ctrlKey: true, altKey: true }),
      repeated: isRunShortcut({ ...key, ctrlKey: true, repeat: true }),
      composing: isRunShortcut({ ...key, ctrlKey: true, isComposing: true }),
    }).toStrictEqual({
      ctrl: true,
      cmd: true,
      plain: false,
      escape: false,
      ctrlCmd: false,
      ctrlShift: false,
      ctrlAlt: false,
      repeated: false,
      composing: false,
    });
  });
});
