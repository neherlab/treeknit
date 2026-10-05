import { describe, expect, test } from "vitest";

import { exampleArgView, exampleConstellation, examplePairView } from "../../drawing/__tests__/fixtures";
import { type InspectorData, inspectorSubject, leafPairs } from "../subject";

const PAIR = examplePairView();

const DATA: InspectorData = { pair: PAIR, arg: exampleArgView(), constellation: exampleConstellation() };

function kindOf(selection: Parameters<typeof inspectorSubject>[0], data = DATA) {
  return inspectorSubject(selection, data).kind;
}

describe("inspectorSubject", () => {
  test("lists the MCCs of the pair when nothing is selected", () => {
    expect(inspectorSubject({}, DATA)).toStrictEqual({ kind: "none", mccs: PAIR.mccs });
  });

  test("shows a selected MCC", () => {
    expect(inspectorSubject({ mcc: 1 }, DATA)).toStrictEqual({ kind: "mcc", mcc: PAIR.mccs[1] });
  });

  test("shows a leaf in both trees with its MCC, its attachment, and its MCC in every pair", () => {
    expect(inspectorSubject({ leaf: "X" }, DATA)).toStrictEqual({
      kind: "leaf",
      name: "X",
      copies: [
        { side: "left", tree: "ha", node: PAIR.left.nodes[8] },
        { side: "right", tree: "na", node: PAIR.right.nodes[5] },
      ],
      mcc: PAIR.mccs[1],
      ambiguous: true,
      pairs: [
        { pair: 0, labels: ["ha", "na"], cell: null },
        { pair: 1, labels: ["ha", "mp"], cell: null },
        { pair: 2, labels: ["na", "mp"], cell: null },
      ],
    });
  });

  test("marks a leaf ambiguous only when its own attachment is ambiguous", () => {
    const mccs = PAIR.mccs.map((mcc) => (mcc.index === 1 ? { ...mcc, ambiguousLeaves: [] } : mcc));
    const subject = inspectorSubject({ leaf: "X" }, { ...DATA, pair: { ...PAIR, mccs } });

    expect(subject).toMatchObject({ kind: "leaf", ambiguous: false });
  });

  test("shows a node of a tree with its clade size and MCC", () => {
    expect(inspectorSubject({ mcc: 0, node: { side: "right", name: "NODE_3" } }, DATA)).toStrictEqual({
      kind: "node",
      side: "right",
      tree: "na",
      node: PAIR.right.nodes[6],
      cladeSize: 2,
      mcc: PAIR.mccs[0],
    });
  });

  test("shows an ARG node", () => {
    expect(inspectorSubject({ node: { side: "arg", name: "H" } }, DATA)).toStrictEqual({
      kind: "argNode",
      node: exampleArgView().nodes[4],
    });
  });

  test.each([
    ["a node wins over a leaf and an MCC", { mcc: 0, leaf: "A", node: { side: "left", name: "NODE_2" } }, "node"],
    ["a leaf wins over an MCC", { mcc: 0, leaf: "A" }, "leaf"],
    ["an unknown node falls back to the leaf", { leaf: "A", node: { side: "left", name: "nope" } }, "leaf"],
    ["an unknown MCC shows nothing", { mcc: 9 }, "none"],
  ] as const)("%s", (_, selection, kind) => {
    expect(kindOf(selection)).toBe(kind);
  });

  test("waits for the ARG before showing an ARG node", () => {
    expect(kindOf({ node: { side: "arg", name: "H" } }, { ...DATA, arg: undefined })).toBe("none");
  });
});

describe("leafPairs", () => {
  test("gives the MCC of a leaf in every pair and none where the pair lacks it", () => {
    expect(leafPairs(exampleConstellation(), "C")).toStrictEqual([
      { pair: 0, labels: ["ha", "na"], cell: { mcc: 0, size: 3, slot: 0 } },
      { pair: 1, labels: ["ha", "mp"], cell: null },
      { pair: 2, labels: ["na", "mp"], cell: null },
    ]);
  });

  test("gives no pairs before the table loads", () => {
    expect(leafPairs(undefined, "C")).toStrictEqual([]);
  });
});
