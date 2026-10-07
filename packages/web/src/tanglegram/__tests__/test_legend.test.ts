import { describe, expect, test } from "vitest";

import type { DrawingColors } from "../../canvas/drawingColors";
import { exampleDrawingRules } from "../../drawing/__tests__/fixtures";
import type { SymbolMark } from "../../drawing/Legend";
import { tanglegramLegend } from "../legend";

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
  mccNone: [131, 144, 141, 255],
};

function marksOf(label: string, colorByMcc: boolean, ribbons: boolean): readonly SymbolMark[] {
  return (
    tanglegramLegend({ colors: COLORS, rules: exampleDrawingRules(), colorByMcc, ribbons }).find(
      (entry) => entry.label === label,
    )?.marks ?? []
  );
}

describe("tanglegramLegend", () => {
  test("draws the imputed ring in the MCC color while branches are colored by MCC", () => {
    expect(marksOf("Imputed leaf", true, true).map((mark) => mark.color)).toStrictEqual([COLORS.mcc[0], COLORS.mcc[0]]);
  });

  test("draws the imputed ring in ink-muted while branches are not colored by MCC", () => {
    expect(marksOf("Imputed leaf", false, true).map((mark) => mark.color)).toStrictEqual([
      COLORS.inkMuted,
      COLORS.inkMuted,
    ]);
  });

  test("draws a reassortment branch 2 px wide in signal with a signal ring", () => {
    expect(marksOf("Reassortment branch", true, true)).toMatchObject([
      { kind: "line", color: COLORS.signal, widthPx: 2 },
      { kind: "ring", color: COLORS.signal, fill: COLORS.ground },
    ]);
  });

  test("draws an added node's branch solid in ink-muted", () => {
    const marks = marksOf("Node added by resolution", true, true);

    expect(marks).toMatchObject([{ kind: "line", color: COLORS.inkMuted }]);
    expect(marks[0]).not.toHaveProperty("dashPx");
  });

  test("shows a ribbon at 55% opacity below 6 px per row, and a link line above", () => {
    expect({
      ribbon: marksOf("Leaves of one MCC", true, true),
      link: marksOf("Leaf of one MCC in both trees", true, false),
    }).toMatchObject({
      ribbon: [{ kind: "area", color: [47, 75, 154, 140] }],
      link: [{ kind: "line", color: COLORS.mcc[0], widthPx: 1 }],
    });
  });
});
