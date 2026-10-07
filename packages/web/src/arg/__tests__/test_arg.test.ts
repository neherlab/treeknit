import { describe, expect, test } from "vitest";

import type { Rgba } from "../../canvas/color";
import type { DrawingColors } from "../../canvas/drawingColors";
import { columnPixel } from "../../canvas/projection";
import {
  exampleArgView,
  exampleDrawingRules,
  LONG_LEAF_NAME,
  LONG_LEAF_SHORT_NAME,
} from "../../drawing/__tests__/fixtures";
import { argEmphasis } from "../../drawing/selection";
import { ARG_LAYER, argColumn, argTargetAt } from "../geometry";
import { argEdgeColor, argSelectionPositions } from "../layers";
import { argLegend } from "../legend";
import { ARG_MISSING, argFailure } from "../outcome";
import { argTargetRows } from "../picking";
import { argGeometry } from "./geometry";

const VIEW = exampleArgView();

const RULES = exampleDrawingRules();

const COLUMN = argColumn(1000, 100, RULES);

const GEOMETRY = argGeometry(VIEW, COLUMN, "y", 64);

const SEGMENT_COLORS: Pick<DrawingColors, "segmentA" | "segmentB" | "ink"> = {
  segmentA: [62, 106, 138, 255],
  segmentB: [138, 106, 62, 255],
  ink: [31, 43, 48, 255],
};

describe("argEdgeColor", () => {
  test.each([
    ["segment A only", [0], SEGMENT_COLORS.segmentA],
    ["segment B only", [1], SEGMENT_COLORS.segmentB],
    ["both segments", [0, 1], SEGMENT_COLORS.ink],
  ] as const)("an edge of %s takes its color", (_, segments, color) => {
    expect(argEdgeColor(segments, SEGMENT_COLORS)).toStrictEqual(color);
  });
});

describe("argGeometry", () => {
  test("separates the dashed reticulation edges from the elbows", () => {
    expect({
      edges: GEOMETRY.edges.map(({ edge }) => edge),
      reticulations: GEOMETRY.reticulations.map(({ edge }) => edge),
    }).toStrictEqual({ edges: [0, 1, 2, 3, 5], reticulations: [4] });
  });

  test("runs a reticulation curve from its parent to the hybrid child", () => {
    const curve = GEOMETRY.reticulations[0]?.path;

    expect([curve?.[0], curve?.at(-1)]).toStrictEqual([
      [columnPixel(COLUMN, 0.5), 0.5],
      [columnPixel(COLUMN, 0.75), 2.5],
    ]);
  });

  test("rings the hybrid node", () => {
    expect(GEOMETRY.hybrids).toStrictEqual([{ node: 4, position: [columnPixel(COLUMN, 0.75), 2.5] }]);
  });

  test("ends each leader at the label edge of the tree column", () => {
    expect(GEOMETRY.leaders.map(({ node, path }) => [node, path.at(-1)])).toStrictEqual([
      [2, [COLUMN.end, 0]],
      [3, [COLUMN.end, 1]],
      [5, [COLUMN.end, 2]],
    ]);
  });

  test("puts the leaf labels at the end of the tree column", () => {
    expect(GEOMETRY.labels.map(({ text, position }) => [text, position])).toStrictEqual([
      ["A", [COLUMN.end, 0]],
      ["B", [COLUMN.end, 1]],
      ["C", [COLUMN.end, 2]],
    ]);
  });
});

describe("argGeometry labels", () => {
  test("draws the short label of each leaf", () => {
    const view = {
      ...VIEW,
      nodes: VIEW.nodes.map((node) =>
        node.label === "B" ? { ...node, label: LONG_LEAF_NAME, shortLabel: LONG_LEAF_SHORT_NAME } : node,
      ),
    };

    expect(argGeometry(view, COLUMN, "y", 64).labels.map(({ text }) => text)).toStrictEqual([
      "A",
      LONG_LEAF_SHORT_NAME,
      "C",
    ]);
  });
});

describe("argLegend", () => {
  test("draws reassortment as dashed edges in their segment colors into a signal ring", () => {
    const signal: Rgba = [176, 38, 94, 255];

    const colors: DrawingColors = {
      ...SEGMENT_COLORS,
      signal,
      ground: SEGMENT_COLORS.ink,
      inkMuted: SEGMENT_COLORS.ink,
      focus: SEGMENT_COLORS.ink,
      mcc: [],
      mccNone: SEGMENT_COLORS.ink,
    };

    const legend = argLegend(colors, RULES, ["ha", "na"]);

    expect({ labels: legend.map(({ label }) => label), reassortment: legend.at(-1)?.marks }).toMatchObject({
      labels: ["Segment ha", "Segment na", "Both segments", "Reassortment"],
      reassortment: [
        { kind: "line", color: SEGMENT_COLORS.segmentA, dashPx: [4, 3] },
        { kind: "line", color: SEGMENT_COLORS.segmentB, dashPx: [4, 3] },
        { kind: "ring", color: signal },
      ],
    });
  });
});

describe("argTargetRows", () => {
  test("zooms to the leaves under the child of an edge and under a node", () => {
    expect({
      edge: argTargetRows(VIEW, { kind: "edge", edge: 0 }),
      node: argTargetRows(VIEW, { kind: "node", node: 4 }),
      missing: argTargetRows(VIEW, { kind: "edge", edge: 9 }),
    }).toStrictEqual({ edge: { first: 0, last: 1 }, node: { first: 2, last: 2 }, missing: null });
  });
});

describe("argTargetAt", () => {
  test.each([
    [ARG_LAYER.edges, 4, { kind: "edge", edge: 5 }],
    [ARG_LAYER.reticulations, 0, { kind: "edge", edge: 4 }],
    [ARG_LAYER.hybrids, 0, { kind: "node", node: 4 }],
    [ARG_LAYER.labels, 1, { kind: "node", node: 3 }],
    ["selection", 0, undefined],
  ] as const)("a pick on %s at %i targets %o", (layer, index, target) => {
    expect(argTargetAt(GEOMETRY, layer, index)).toStrictEqual(target);
  });
});

describe("argSelectionPositions", () => {
  test("rings the selected leaf and the selected hybrid node", () => {
    expect(
      argSelectionPositions(GEOMETRY, argEmphasis(VIEW, { leaf: "A", node: { side: "arg", name: "H" } })),
    ).toStrictEqual([GEOMETRY.nodes[2], GEOMETRY.nodes[4]]);
  });
});

describe("argFailure", () => {
  test.each([
    ["a built ARG", { status: "built", reassortments: 1 }, VIEW, undefined],
    ["an ARG still loading", { status: "built", reassortments: 1 }, undefined, undefined],
    ["a failed ARG", { status: "failed", message: "The trees share no MCC." }, null, "The trees share no MCC"],
    ["a missing ARG", null, null, ARG_MISSING],
  ] as const)("%s gives its notice", (_, outcome, view, failure) => {
    expect(argFailure(outcome, view)).toBe(failure);
  });
});
