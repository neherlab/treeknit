import { describe, expect, test } from "vitest";

import { argTooltip, pairTooltip, segmentLabels, segmentList } from "../tooltip";
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

  test("marks an imputed leaf and a node added by resolution", () => {
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
        "Added by resolution or imputation",
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

describe("segmentLabels", () => {
  test("names the segments after the first two trees, or A and B without them", () => {
    expect([segmentLabels([{ label: "ha" }, { label: "na" }, { label: "mp" }]), segmentLabels([])]).toStrictEqual([
      ["ha", "na"],
      ["A", "B"],
    ]);
  });
});

describe("argTooltip", () => {
  test("shows the segments and the branch length per segment of a hybrid node", () => {
    expect(argTooltip(exampleArgView(), { kind: "node", node: 4 }, SEGMENTS)).toStrictEqual([
      "H",
      "Segments ha and na",
      "Branch length in ha: 0.1",
      "Branch length in na: 0.2",
      "Hybrid node",
    ]);
  });

  test("shows the segment of a reticulation edge", () => {
    expect(argTooltip(exampleArgView(), { kind: "edge", edge: 4 }, SEGMENTS)).toStrictEqual([
      "Segment na",
      "Reticulation edge",
    ]);
  });

  test("names one segment in the singular", () => {
    expect(segmentList([0], SEGMENTS)).toBe("Segment ha");
  });
});
