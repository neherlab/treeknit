import { describe, expect, test } from "vitest";

import type { DrawingColors } from "../../canvas/drawingColors";
import { columnPixel } from "../../canvas/projection";
import { exampleArgView } from "../../drawing/__tests__/fixtures";
import { argEmphasis } from "../../drawing/selection";
import { DRAWING_MARGIN_PX, LABEL_GAP_PX } from "../../drawing/spacing";
import { ARG_LABEL_MAX_SHARE, ARG_LAYER, argColumn, argGeometry, argTargetAt } from "../geometry";
import { argEdgeColor, argSelectionPositions } from "../layers";
import { ARG_MISSING, argFailure } from "../outcome";

const VIEW = exampleArgView();

const COLUMN = argColumn(1000, 100);

const GEOMETRY = argGeometry(VIEW, COLUMN, "y");

const SEGMENT_COLORS: Pick<DrawingColors, "segmentA" | "segmentB" | "ink"> = {
  segmentA: [62, 106, 138, 255],
  segmentB: [138, 106, 62, 255],
  ink: [31, 43, 48, 255],
};

describe("argColumn", () => {
  test("leaves room for the labels at the right", () => {
    expect(COLUMN).toStrictEqual({
      start: DRAWING_MARGIN_PX,
      end: 1000 - DRAWING_MARGIN_PX - (100 + 2 * LABEL_GAP_PX),
      mirrored: false,
    });
  });

  test("gives labels at most a quarter of the width", () => {
    expect(argColumn(1000, 900).end).toBe(1000 - DRAWING_MARGIN_PX - ARG_LABEL_MAX_SHARE * 1000);
  });
});

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

  test("puts the leaf labels at the end of the tree column", () => {
    expect(GEOMETRY.labels.map(({ name, position }) => [name, position])).toStrictEqual([
      ["A", [COLUMN.end, 0]],
      ["B", [COLUMN.end, 1]],
      ["C", [COLUMN.end, 2]],
    ]);
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
