import { describe, expect, test } from "vitest";

import type { DrawingColors } from "../../canvas/drawingColors";
import type { SymbolMark } from "../Legend";
import { legendEntries } from "../legendMarks";
import { exampleArgView, exampleDrawingRules, examplePairLegend } from "./fixtures";

const COLORS: DrawingColors = {
  ground: [243, 245, 244, 255],
  ink: [31, 43, 48, 255],
  inkMuted: [85, 101, 107, 255],
  signal: [176, 38, 94, 255],
  focus: [36, 87, 197, 255],
  segmentA: [62, 106, 138, 255],
  segmentB: [138, 106, 62, 255],
  mcc: [
    [47, 75, 154, 255],
    [147, 119, 28, 255],
    [44, 140, 131, 255],
    [124, 79, 172, 255],
    [122, 85, 55, 255],
    [103, 118, 48, 255],
    [206, 121, 30, 255],
    [91, 124, 153, 255],
  ],
  noMcc: [131, 144, 141, 255],
};

const RULES = exampleDrawingRules();

function marksOf(label: string, colorByMcc: boolean, ribbons: boolean): readonly SymbolMark[] {
  return (
    legendEntries(examplePairLegend(), { colors: COLORS, rules: RULES, colorByMcc, ribbons }).find(
      (entry) => entry.label === label,
    )?.marks ?? []
  );
}

describe("legendEntries", () => {
  test("keeps the labels of the legend of the display data", () => {
    const style = { colors: COLORS, rules: RULES, colorByMcc: true, ribbons: false };

    expect(legendEntries(examplePairLegend(), style).map(({ label }) => label)).toStrictEqual([
      "Reassortment branch",
      "Node added by resolution or imputation",
      "Imputed leaf",
      "Leaves of one MCC",
      "No MCC",
    ]);
  });

  test("draws the imputed branch across the symbol and its ring at three quarters, in the first MCC color while branches are colored by MCC", () => {
    expect(marksOf("Imputed leaf", true, true)).toMatchObject([
      { kind: "line", path: "M0 6 H24", color: COLORS.mcc[0] },
      { kind: "ring", at: [18, 6], color: COLORS.mcc[0] },
    ]);
  });

  test("draws the imputed branch and ring in ink-muted while branches are not colored by MCC", () => {
    expect(marksOf("Imputed leaf", false, true).map((mark) => mark.color)).toStrictEqual([
      COLORS.inkMuted,
      COLORS.inkMuted,
    ]);
  });

  test("draws a reassortment branch 2 px wide in signal with a signal ring at its middle", () => {
    expect(marksOf("Reassortment branch", true, true)).toMatchObject([
      { kind: "line", color: COLORS.signal, widthPx: 2 },
      { kind: "ring", at: [12, 6], color: COLORS.signal, fill: COLORS.ground },
    ]);
  });

  test("draws an added node's branch as a solid ink-muted line, as the drawing draws its part across", () => {
    expect(marksOf("Node added by resolution or imputation", true, true)).toStrictEqual([
      { kind: "line", path: "M0 6 H24", color: COLORS.inkMuted, widthPx: 1.5 },
    ]);
  });

  test("shows the links as a ribbon at 55% opacity below 6 px per row, and as a link line above", () => {
    expect({
      ribbon: marksOf("Leaves of one MCC", true, true),
      link: marksOf("Leaves of one MCC", true, false),
    }).toMatchObject({
      ribbon: [{ kind: "area", color: [47, 75, 154, 140] }],
      link: [{ kind: "line", color: COLORS.mcc[0], widthPx: 1 }],
    });
  });

  test("draws ARG reassortment as a dashed curve in the color of its reticulation edge into a signal ring", () => {
    const style = { colors: COLORS, rules: RULES, colorByMcc: true, ribbons: false };

    expect(legendEntries(exampleArgView().legend, style).at(-1)).toMatchObject({
      label: "Reassortment",
      marks: [
        { kind: "line", color: COLORS.segmentB, dashPx: [4, 3] },
        { kind: "ring", at: [24, 6], color: COLORS.signal },
      ],
    });
  });
});
