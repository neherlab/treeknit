import type { Overlap, PairSummary, Summary } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { noReassortmentFound, overlapMatrix, pipelinePairs, resultOverview } from "../overviewModel";

describe("overview after a run", () => {
  test("two trees: one row and the ARG outcome, without a matrix", () => {
    const summary = summaryOf([pair(0, ["ha", "na"], 2, 1, 0)], { status: "built", reassortments: 1 });

    expect(resultOverview(summary, ["ha", "na"])).toStrictEqual({
      rows: [{ index: 0, labels: ["ha", "na"], mccCount: 2, imputedCount: 1, ambiguousCount: 0 }],
      arg: { status: "built", reassortments: 1 },
      noReassortment: false,
      matrix: null,
    });
  });

  test("three trees: one row per pair and a symmetric matrix of MCC counts in pipeline order", () => {
    const summary = summaryOf(
      [pair(0, ["ha", "na"], 2, 0, 0), pair(1, ["ha", "pb2"], 3, 2, 1), pair(2, ["na", "pb2"], 4, 0, 0)],
      null,
    );

    const overview = resultOverview(summary, ["ha", "na", "pb2"]);

    expect({
      rows: overview.rows.map(({ labels, mccCount }) => [...labels, mccCount]),
      arg: overview.arg,
      matrix: overview.matrix?.rows.map((row) => row.map(({ value }) => value?.mccCount ?? null)),
      links: overview.matrix?.rows.map((row) => row.map(({ value }) => value?.pair ?? null)),
    }).toStrictEqual({
      rows: [
        ["ha", "na", 2],
        ["ha", "pb2", 3],
        ["na", "pb2", 4],
      ],
      arg: null,
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

  test("lists pairs in the pipeline order of the core", () => {
    expect(pipelinePairs(4)).toStrictEqual([
      [0, 1],
      [0, 2],
      [0, 3],
      [1, 2],
      [1, 3],
      [2, 3],
    ]);
  });
});

describe("no reassortment found", () => {
  test("two trees with a built ARG of zero reassortments", () => {
    expect(noReassortmentFound(summaryOf([pair(0, ["ha", "na"], 1, 0, 0)], built(0)), 2)).toBe(true);
  });

  test("not for two trees with one MCC whose ARG failed", () => {
    const failed = summaryOf([pair(0, ["ha", "na"], 1, 3, 0)], { status: "failed", message: "leaf X is missing" });

    expect(noReassortmentFound(failed, 2)).toBe(false);
  });

  test("not for two trees with reassortments", () => {
    expect(noReassortmentFound(summaryOf([pair(0, ["ha", "na"], 2, 0, 0)], built(1)), 2)).toBe(false);
  });

  test("more than two trees only when every pair has exactly one MCC", () => {
    const one = [pair(0, ["a", "b"], 1, 0, 0), pair(1, ["a", "c"], 1, 0, 0), pair(2, ["b", "c"], 1, 0, 0)];
    const split = [pair(0, ["a", "b"], 1, 0, 0), pair(1, ["a", "c"], 2, 0, 0), pair(2, ["b", "c"], 1, 0, 0)];

    expect({
      one: noReassortmentFound(summaryOf(one, null), 3),
      split: noReassortmentFound(summaryOf(split, null), 3),
    }).toStrictEqual({ one: true, split: false });
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
  labels: [string, string],
  mccCount: number,
  imputedCount: number,
  ambiguousCount: number,
): PairSummary {
  return {
    index,
    labels,
    mccCount,
    mccs: Array.from({ length: mccCount }, (_, mcc) => [`L${String(mcc)}`]),
    imputedCount,
    ambiguousCount,
  };
}

function built(reassortments: number): Summary["arg"] {
  return { status: "built", reassortments };
}

function summaryOf(pairs: PairSummary[], arg: Summary["arg"]): Summary {
  return { pairs, arg, diagnostics: [] };
}
