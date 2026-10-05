import type { ValidationError } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { fieldMessage, groupFieldErrors, seqLengthField, shownFields, treeField } from "../fieldErrors";

describe("validation errors by field", () => {
  test("puts errors on the fields the workspace shows and the rest in the general list", () => {
    const errors = [
      error("settings.gamma", "gamma must be finite and not negative"),
      error("trees[1].label", "label is empty"),
      error("trees[1].label", "label repeats"),
      error("settings.seqLengths[0]", "sequence length must be positive"),
      error("trees", "trees ha and na share fewer than two leaves"),
      error(null, "need at least two trees"),
      error("settings.seqLengths", "one sequence length per tree"),
    ];

    const grouped = groupFieldErrors(errors, shownFields(2, false));

    expect({
      gamma: fieldMessage(grouped, "settings.gamma"),
      label: fieldMessage(grouped, treeField(1, "label")),
      list: fieldMessage(grouped, "trees"),
      newick: fieldMessage(grouped, treeField(0, "newick")),
      general: grouped.general.map(({ message }) => message),
    }).toStrictEqual({
      gamma: "gamma must be finite and not negative",
      label: "label is empty; label repeats",
      list: "trees ha and na share fewer than two leaves",
      newick: undefined,
      general: ["sequence length must be positive", "need at least two trees", "one sequence length per tree"],
    });
  });

  test("shows sequence-length errors on the tree rows while sequence lengths are on", () => {
    const grouped = groupFieldErrors([error("settings.seqLengths[1]", "must be positive")], shownFields(2, true));

    expect({ field: fieldMessage(grouped, seqLengthField(1)), general: grouped.general }).toStrictEqual({
      field: "must be positive",
      general: [],
    });
  });
});

function error(field: string | null, message: string): ValidationError {
  return { field, message, line: null, column: null };
}
