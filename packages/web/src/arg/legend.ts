import type { DrawingRules } from "@neherlab/treeknit-wasm";

import type { DrawingColors } from "../canvas/drawingColors";
import type { LegendEntry, SymbolMark } from "../drawing/Legend";
import { horizontalPath, SYMBOL_HEIGHT_PX, SYMBOL_MIDDLE_PX, sCurvePath } from "../drawing/legendSymbols";
import type { SegmentLabels } from "../drawing/tooltip";
import { argEdgeColor } from "./layers";

const STRAIGHT = horizontalPath();

const RETICULATION_END_PX = 16;

const RETICULATION_CONTROL_PX = 9;

const FROM_SEGMENT_A = sCurvePath(1, SYMBOL_MIDDLE_PX, RETICULATION_END_PX, RETICULATION_CONTROL_PX);

const FROM_SEGMENT_B = sCurvePath(SYMBOL_HEIGHT_PX - 1, SYMBOL_MIDDLE_PX, RETICULATION_END_PX, RETICULATION_CONTROL_PX);

const SEGMENT_A = [0];

const SEGMENT_B = [1];

const BOTH_SEGMENTS = [0, 1];

export function argLegend(colors: DrawingColors, rules: DrawingRules, [a, b]: SegmentLabels): LegendEntry[] {
  const edge = (segments: readonly number[], path: string, dashed: boolean): SymbolMark => ({
    kind: "line",
    path,
    color: argEdgeColor(segments, colors),
    widthPx: rules.branchWidthPx,
    ...(dashed ? { dashPx: rules.dashPx } : undefined),
  });

  return [
    { label: `Segment ${a}`, marks: [edge(SEGMENT_A, STRAIGHT, false)] },
    { label: `Segment ${b}`, marks: [edge(SEGMENT_B, STRAIGHT, false)] },
    { label: "Both segments", marks: [edge(BOTH_SEGMENTS, STRAIGHT, false)] },
    {
      label: "Reassortment",
      marks: [
        edge(SEGMENT_A, FROM_SEGMENT_A, true),
        edge(SEGMENT_B, FROM_SEGMENT_B, true),
        {
          kind: "ring",
          at: [19, SYMBOL_MIDDLE_PX],
          color: colors.signal,
          fill: colors.ground,
          radiusPx: rules.markRadiusPx,
          lineWidthPx: rules.markLinePx,
        },
      ],
    },
  ];
}
