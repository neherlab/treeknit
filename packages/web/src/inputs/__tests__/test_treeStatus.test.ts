import type { TreeInspection, ValidationError } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { treeStatus } from "../treeStatus";

const PARSED: TreeInspection = {
  label: "ha",
  leaves: 5,
  internalNodes: 4,
  polytomies: 0,
  branchLengths: "all",
  warnings: [],
  error: null,
};

const DUPLICATE: ValidationError = {
  field: "trees[0].newick",
  message: 'tree "ha": duplicate leaf name A',
  line: null,
  column: null,
};

describe("tree status", () => {
  test("a parse error marks the error column", () => {
    const broken: TreeInspection = {
      ...PARSED,
      leaves: 0,
      error: { message: "expected ')'", line: 2, column: 5 },
    };

    expect(treeStatus(broken, [DUPLICATE])).toStrictEqual({
      kind: "error",
      messages: ["expected ')'"],
      errorRange: { start: { line: 2, column: 5 }, end: { line: 2, column: 6 } },
    });
  });

  test("an error without a position from validation", () => {
    expect(treeStatus(PARSED, [DUPLICATE])).toStrictEqual({
      kind: "error",
      messages: ['tree "ha": duplicate leaf name A'],
      errorRange: undefined,
    });
  });

  test("parser warnings", () => {
    const warned = { ...PARSED, warnings: ["more than one tree in file, using the first"] };

    expect(treeStatus(warned, [])).toStrictEqual({
      kind: "warning",
      messages: ["more than one tree in file, using the first"],
      errorRange: undefined,
    });
  });

  test("ready, and pending before the inspection arrives", () => {
    expect({ ready: treeStatus(PARSED, []).kind, pending: treeStatus(undefined, []).kind }).toStrictEqual({
      ready: "ok",
      pending: "pending",
    });
  });
});
