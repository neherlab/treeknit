import { useMemo } from "react";

import type { DrawingColors } from "../canvas/drawingColors";
import { Legend } from "../drawing/Legend";
import type { SegmentLabels } from "../drawing/tooltip";
import { argLegend } from "./legend";

export function ArgLegend({ colors, segments }: { colors: DrawingColors; segments: SegmentLabels }) {
  const entries = useMemo(() => argLegend(colors, segments), [colors, segments]);

  return <Legend entries={entries} />;
}
