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
  field: { kind: "treeNewick", index: 0 },
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
      messages: [{ severity: "error", text: "expected ')'" }],
      errorRange: { start: { line: 2, column: 5 }, end: { line: 2, column: 6 } },
    });
  });

  test("an error without a position from validation", () => {
    expect(treeStatus(PARSED, [DUPLICATE])).toStrictEqual({
      kind: "error",
      messages: [{ severity: "error", text: 'tree "ha": duplicate leaf name A' }],
      errorRange: undefined,
    });
  });

  test("a malformed first tree followed by a second tree keeps both the parse error and the several-trees warning", () => {
    const broken: TreeInspection = {
      ...PARSED,
      leaves: 0,
      warnings: ["more than one tree in file, using the first"],
      error: { message: "expected ')'", line: 1, column: 6 },
    };

    expect(treeStatus(broken, [])).toStrictEqual({
      kind: "error",
      messages: [
        { severity: "error", text: "expected ')'" },
        { severity: "warning", text: "more than one tree in file, using the first" },
      ],
      errorRange: { start: { line: 1, column: 6 }, end: { line: 1, column: 7 } },
    });
  });

  test("a validation error keeps the parser warnings", () => {
    const warned = { ...PARSED, warnings: ["more than one tree in file, using the first"] };

    expect(treeStatus(warned, [DUPLICATE]).messages).toStrictEqual([
      { severity: "error", text: 'tree "ha": duplicate leaf name A' },
      { severity: "warning", text: "more than one tree in file, using the first" },
    ]);
  });

  test("parser warnings", () => {
    const warned = { ...PARSED, warnings: ["more than one tree in file, using the first"] };

    expect(treeStatus(warned, [])).toStrictEqual({
      kind: "warning",
      messages: [{ severity: "warning", text: "more than one tree in file, using the first" }],
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
