import type { DrawingColors } from "../canvas/drawingColors";
import { DASH_PX } from "../canvas/layers/pathLayer";
import { BRANCH_WIDTH_PX, MARK_LINE_PX, MARK_RADIUS_PX } from "../canvas/layers/treeLayers";
import type { LegendEntry, SymbolMark } from "../drawing/Legend";
import type { SegmentLabels } from "../drawing/tooltip";
import { argEdgeColor } from "./layers";

const STRAIGHT = "M0 6 H24";

const FROM_SEGMENT_A = "M0 1 C9 1 9 6 16 6";

const FROM_SEGMENT_B = "M0 11 C9 11 9 6 16 6";

const SEGMENT_A = [0];

const SEGMENT_B = [1];

const BOTH_SEGMENTS = [0, 1];

export function argLegend(colors: DrawingColors, [a, b]: SegmentLabels): LegendEntry[] {
  const edge = (segments: readonly number[], path: string, dashed: boolean): SymbolMark => ({
    kind: "line",
    path,
    color: argEdgeColor(segments, colors),
    widthPx: BRANCH_WIDTH_PX,
    ...(dashed ? { dashPx: DASH_PX } : undefined),
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
          at: [19, 6],
          color: colors.signal,
          fill: colors.ground,
          radiusPx: MARK_RADIUS_PX,
          lineWidthPx: MARK_LINE_PX,
        },
      ],
    },
  ];
}
