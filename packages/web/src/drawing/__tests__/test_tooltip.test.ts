import { describe, expect, test } from "vitest";

import { argTooltip, pairTooltip, segmentList } from "../tooltip";
import { exampleArgView, examplePairView } from "./fixtures";

const VIEW = examplePairView();

const SEGMENTS = ["ha", "na"] as const;

describe("pairTooltip", () => {
  test("names a reassortment leaf with its MCC and branch length", () => {
    expect(pairTooltip(VIEW, { kind: "node", side: "left", node: 8 })).toStrictEqual([
      "X",
      "MCC 2, 1 leaf",
      "Branch length: 1",
      "Reassortment branch",
    ]);
  });

  test("marks an imputed leaf and a node added by resolution or imputation with the legend labels", () => {
    expect({
      imputed: pairTooltip(VIEW, { kind: "node", side: "right", node: 5 }).at(-1),
      added: pairTooltip(VIEW, { kind: "node", side: "right", node: 3 }),
    }).toStrictEqual({
      imputed: "Imputed leaf",
      added: [
        "RESOLVED_1",
        "No MCC",
        "Branch length: 0",
        "Drawn at the mean length of the trees: 0.25",
        "Node added by resolution or imputation",
      ],
    });
  });

  test("describes a link and a ribbon by their MCC", () => {
    expect({
      link: pairTooltip(VIEW, { kind: "link", link: 0 }),
      ribbon: pairTooltip(VIEW, { kind: "ribbon", block: 1 }),
    }).toStrictEqual({
      link: ["A", "MCC 1, 4 leaves"],
      ribbon: ["MCC 1, 4 leaves", "2 leaves in this block"],
    });
  });
});

describe("argTooltip", () => {
  test("shows both segments, the branch length per segment of the view's trees, and reassortment at a hybrid node", () => {
    expect(argTooltip(exampleArgView(), { kind: "node", node: 4 })).toStrictEqual([
      "H",
      "Both segments",
      "Branch length in ha: 0.1",
      "Branch length in na: 0.2",
      "Reassortment",
    ]);
  });

  test("shows the segment of a reticulation edge", () => {
    expect(argTooltip(exampleArgView(), { kind: "edge", edge: 4 })).toStrictEqual(["Segment na", "Reticulation edge"]);
  });

  test("names one segment after its tree, and two segments as both", () => {
    expect([segmentList([0], SEGMENTS), segmentList([1], SEGMENTS), segmentList([0, 1], SEGMENTS)]).toStrictEqual([
      "Segment ha",
      "Segment na",
      "Both segments",
    ]);
  });
});
