import type { Field, ValidationError } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { fieldErrorsAt, fieldMessage, groupFieldErrors, shownFields } from "../fieldErrors";

const GAMMA: Field = { kind: "setting", key: "gamma" };

const SEQ_LENGTHS: Field = { kind: "setting", key: "seqLengths" };

const TREES: Field = { kind: "trees" };

describe("validation errors by field", () => {
  test("puts errors on the fields the workspace shows and the rest in the general list", () => {
    const errors = [
      error(GAMMA, "gamma must be finite and not negative"),
      error({ kind: "treeLabel", index: 1 }, "label is empty"),
      error({ kind: "treeLabel", index: 1 }, "label repeats"),
      error({ kind: "seqLength", index: 0 }, "sequence length must be positive"),
      error(TREES, "trees ha and na share fewer than two leaves"),
      error(null, "need at least two trees"),
      error(SEQ_LENGTHS, "one sequence length per tree"),
    ];

    const grouped = groupFieldErrors(errors, shownFields(2, false));

    expect({
      gamma: fieldMessage(grouped, GAMMA),
      label: fieldMessage(grouped, { kind: "treeLabel", index: 1 }),
      list: fieldMessage(grouped, TREES),
      newick: fieldMessage(grouped, { kind: "treeNewick", index: 0 }),
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
    const grouped = groupFieldErrors(
      [error({ kind: "seqLength", index: 1 }, "must be positive")],
      shownFields(2, true),
    );

    expect({ field: fieldMessage(grouped, { kind: "seqLength", index: 1 }), general: grouped.general }).toStrictEqual({
      field: "must be positive",
      general: [],
    });
  });

  test("tells fields of the same kind apart by their index", () => {
    const newick = error({ kind: "treeNewick", index: 1 }, "parse error");
    const grouped = groupFieldErrors([newick], shownFields(2, false));

    expect({
      first: fieldErrorsAt(grouped, { kind: "treeNewick", index: 0 }),
      second: fieldErrorsAt(grouped, { kind: "treeNewick", index: 1 }),
    }).toStrictEqual({ first: [], second: [newick] });
  });

  test("leaves the errors of a link key in the general list", () => {
    const grouped = groupFieldErrors(
      [error({ kind: "linkKey", key: "gamma" }, "gamma must be a number")],
      shownFields(2, false),
    );

    expect({ byField: grouped.byField, general: grouped.general.map(({ message }) => message) }).toStrictEqual({
      byField: [],
      general: ["gamma must be a number"],
    });
  });
});

function error(field: Field | null, message: string): ValidationError {
  return { field, message, line: null, column: null };
}
