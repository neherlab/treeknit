import type { Overlap, PairSummary, Summary } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { overlapMatrix, resultOverview } from "../overviewModel";

describe("overview after a run", () => {
  test("two trees: one row, the ARG outcome, and the no-reassortment result of the summary, without a matrix", () => {
    const summary = summaryOf([pair(0, [0, 1], ["ha", "na"], 1, 1, 0)], { status: "built", reassortments: 0 }, true);

    expect(resultOverview(summary, ["ha", "na"])).toStrictEqual({
      rows: [{ index: 0, labels: ["ha", "na"], mccCount: 1, imputedCount: 1, ambiguousCount: 0 }],
      arg: { status: "built", reassortments: 0 },
      noReassortment: true,
      matrix: null,
    });
  });

  test("three trees: one row per pair and a symmetric matrix of MCC counts at the trees of each pair", () => {
    const summary = summaryOf(
      [
        pair(0, [0, 1], ["ha", "na"], 2, 0, 0),
        pair(1, [0, 2], ["ha", "pb2"], 3, 2, 1),
        pair(2, [1, 2], ["na", "pb2"], 4, 0, 0),
      ],
      null,
      false,
    );

    const overview = resultOverview(summary, ["ha", "na", "pb2"]);

    expect({
      rows: overview.rows.map(({ labels, mccCount }) => [...labels, mccCount]),
      arg: overview.arg,
      noReassortment: overview.noReassortment,
      matrix: overview.matrix?.rows.map((row) => row.map(({ value }) => value?.mccCount ?? null)),
      links: overview.matrix?.rows.map((row) => row.map(({ value }) => value?.pair ?? null)),
    }).toStrictEqual({
      rows: [
        ["ha", "na", 2],
        ["ha", "pb2", 3],
        ["na", "pb2", 4],
      ],
      arg: null,
      noReassortment: false,
      matrix: [
        [null, 2, 3],
        [2, null, 4],
        [3, 4, null],
      ],
      links: [
        [null, 0, 1],
        [0, null, 2],
        [1, 2, null],
      ],
    });
  });

  test("places each pair at its own trees, whatever its position in the list", () => {
    const summary = summaryOf(
      [pair(2, [1, 2], ["na", "pb2"], 4, 0, 0), pair(0, [0, 1], ["ha", "na"], 2, 0, 0)],
      null,
      false,
    );

    expect(
      resultOverview(summary, ["ha", "na", "pb2"]).matrix?.rows.map((row) => row.map(({ value }) => value)),
    ).toStrictEqual([
      [null, { pair: 0, mccCount: 2 }, null],
      [{ pair: 0, mccCount: 2 }, null, { pair: 2, mccCount: 4 }],
      [null, { pair: 2, mccCount: 4 }, null],
    ]);
  });
});

describe("overview before a run", () => {
  test("a symmetric matrix of shared leaves with blocked pairs and an empty cell for a tree that does not parse", () => {
    const overlap: Overlap = {
      totalLeaves: 6,
      trees: [
        { index: 0, label: "ha", leaves: 5, missing: 1 },
        { index: 2, label: "pb2", leaves: 2, missing: 4 },
      ],
      pairs: [{ i: 0, j: 2, shared: 1, blocked: true }],
      failed: [1],
    };

    expect(overlapMatrix(overlap, ["ha", "na", "pb2"]).rows.map((row) => row.map(({ value }) => value))).toStrictEqual([
      [null, null, { shared: 1, blocked: true }],
      [null, null, null],
      [{ shared: 1, blocked: true }, null, null],
    ]);
  });
});

function pair(
  index: number,
  trees: [number, number],
  labels: [string, string],
  mccCount: number,
  imputedCount: number,
  ambiguousCount: number,
): PairSummary {
  return {
    index,
    trees,
    labels,
    mccCount,
    mccs: Array.from({ length: mccCount }, (_, mcc) => [`L${String(mcc)}`]),
    imputedCount,
    ambiguousCount,
  };
}

function summaryOf(pairs: PairSummary[], arg: Summary["arg"], noReassortment: boolean): Summary {
  return { pairs, arg, noReassortment, diagnostics: [] };
}
