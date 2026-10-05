import type { DrawingRules } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import type { DrawingColors } from "../canvas/drawingColors";
import { Legend } from "../drawing/Legend";
import type { SegmentLabels } from "../drawing/tooltip";
import { argLegend } from "./legend";

export function ArgLegend({ colors, rules, segments }: ArgLegendProps) {
  const entries = useMemo(() => argLegend(colors, rules, segments), [colors, rules, segments]);

  return <Legend entries={entries} />;
}

export interface ArgLegendProps {
  colors: DrawingColors;
  rules: DrawingRules;
  segments: SegmentLabels;
}
