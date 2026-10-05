import type { Settings } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import type { UndoEntry } from "../store";
import { undoMessage } from "../undo";

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
  seed: 1,
};

describe("undoMessage", () => {
  test.each<[string, UndoEntry, string]>([
    [
      "a removed tree",
      {
        kind: "tree",
        tree: { id: "t", label: "na", newick: "(A,B);", textId: 1, source: { kind: "paste" } },
        index: 1,
        seqLength: null,
      },
      "Removed na",
    ],
    [
      "a cleared workspace",
      { kind: "workspace", reason: "clear", trees: [], settings: SETTINGS, result: null },
      "Workspace cleared",
    ],
    [
      "an opened session file",
      { kind: "workspace", reason: "session", trees: [], settings: SETTINGS, result: null },
      "Session file opened",
    ],
    ["reset settings", { kind: "settings", settings: SETTINGS }, "Settings reset to defaults"],
  ])("names %s", (_case, entry, message) => {
    expect(undoMessage(entry)).toBe(message);
  });
});
