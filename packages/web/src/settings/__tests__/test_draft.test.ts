import type { Settings } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { selectRequest, type WorkspaceData } from "../../workspace/store";
import {
  draftError,
  draftFromSettings,
  ENTER_A_NUMBER,
  hasDraft,
  nextDraft,
  syncDraft,
  withNumberSetting,
  withSeqLength,
} from "../draft";

const SETTINGS: Settings = {
  gamma: 2,
  seqLengths: null,
  nMcmcIt: 25,
  resolve: "strict",
  preResolve: false,
  rounds: 1,
  finalRound: true,
  likelihood: true,
  naive: false,
  seed: 7,
};

const IDS = ["tree-1", "tree-2", "tree-3"];

describe("settings draft", () => {
  test("mirrors the store settings with sequence lengths keyed by tree id", () => {
    expect(draftFromSettings({ ...SETTINGS, seqLengths: [10, 20, 30] }, IDS)).toStrictEqual({
      gamma: 2,
      nMcmcIt: 25,
      rounds: 1,
      seed: 7,
      seqLengths: { "tree-1": 10, "tree-2": 20, "tree-3": 30 },
    });
  });

  test("maps a parsed number edit to the settings that the request sends", () => {
    const settings = withNumberSetting(SETTINGS, "gamma", 3.5);
    const state: Pick<WorkspaceData, "trees" | "settings"> = { trees: [], settings: settings ?? SETTINGS };

    expect(selectRequest({ ...emptyWorkspace(), ...state }).settings).toStrictEqual({ ...SETTINGS, gamma: 3.5 });
  });

  test("leaves the store at its last valid value while the field holds no number", () => {
    expect({
      empty: withNumberSetting(SETTINGS, "seed", Number.NaN),
      infinite: withNumberSetting(SETTINGS, "rounds", Number.POSITIVE_INFINITY),
      seqLength: withSeqLength({ ...SETTINGS, seqLengths: [1, 2, 3] }, IDS, "tree-2", Number.NaN),
    }).toStrictEqual({ empty: null, infinite: null, seqLength: null });
  });

  test("sets the sequence length of the tree with the given id", () => {
    expect(withSeqLength({ ...SETTINGS, seqLengths: [1, 2, 3] }, IDS, "tree-2", 1700)?.seqLengths).toStrictEqual([
      1, 1700, 3,
    ]);
  });

  test("ignores a sequence length while sequence lengths are off or for an unknown tree", () => {
    expect({
      off: withSeqLength(SETTINGS, IDS, "tree-1", 5),
      unknown: withSeqLength({ ...SETTINGS, seqLengths: [1, 2, 3] }, IDS, "tree-9", 5),
    }).toStrictEqual({ off: null, unknown: null });
  });

  test("keeps an invalid draft through store changes and follows the store everywhere else", () => {
    const draft = { ...draftFromSettings({ ...SETTINGS, seqLengths: [1, 2, 3] }, IDS), gamma: Number.NaN };
    const withDraftLength = { ...draft, seqLengths: { ...draft.seqLengths, "tree-3": Number.NaN } };
    const reordered = ["tree-3", "tree-1"];

    expect(syncDraft(withDraftLength, { ...SETTINGS, seed: 9, seqLengths: [30, 10] }, reordered)).toStrictEqual({
      gamma: Number.NaN,
      nMcmcIt: 25,
      rounds: 1,
      seed: 9,
      seqLengths: { "tree-3": Number.NaN, "tree-1": 10 },
    });
  });

  test("the sequence-length switch adds and removes the per-tree fields", () => {
    const off = draftFromSettings(SETTINGS, IDS);
    const on = syncDraft(off, { ...SETTINGS, seqLengths: [1, 1, 1] }, IDS);

    expect({ on: on.seqLengths, offAgain: syncDraft(on, SETTINGS, IDS).seqLengths }).toStrictEqual({
      on: { "tree-1": 1, "tree-2": 1, "tree-3": 1 },
      offAgain: {},
    });
  });

  test("reports a draft and its message only for a field without a number", () => {
    const clean = draftFromSettings({ ...SETTINGS, seqLengths: [1, 2, 3] }, IDS);
    const lengthDraft = { ...clean, seqLengths: { ...clean.seqLengths, "tree-2": Number.NaN } };

    expect({
      clean: hasDraft(clean),
      gamma: hasDraft({ ...clean, gamma: Number.NaN }),
      length: hasDraft(lengthDraft),
      message: draftError(Number.NaN),
      noMessage: draftError(0),
    }).toStrictEqual({ clean: false, gamma: true, length: true, message: ENTER_A_NUMBER, noMessage: undefined });
  });

  test("discards drafts when the workspace was replaced, and keeps them through other changes", () => {
    const draft = { ...draftFromSettings(SETTINGS, IDS), seed: Number.NaN };
    const loaded = { ...SETTINGS, gamma: 5, seed: 11 };

    expect({
      edited: nextDraft(draft, loaded, IDS, false).seed,
      replaced: nextDraft(draft, loaded, IDS, true),
    }).toStrictEqual({
      edited: Number.NaN,
      replaced: { gamma: 5, nMcmcIt: 25, rounds: 1, seed: 11, seqLengths: {} },
    });
  });
});

function emptyWorkspace(): WorkspaceData {
  return {
    trees: [],
    settings: SETTINGS,
    run: { status: "idle" },
    result: null,
    defaults: SETTINGS,
    undo: null,
    restored: false,
    nextTreeNumber: 1,
    nextTextId: 1,
    resetRevision: 0,
  };
}
