import type { DrawingRules, LegendItem } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import type { DrawingColors } from "../canvas/drawingColors";
import { Legend } from "../drawing/Legend";
import { legendEntries } from "../drawing/legendMarks";

export function ArgLegend({ items, colors, rules }: ArgLegendProps) {
  const entries = useMemo(
    () => legendEntries(items, { colors, rules, colorByMcc: true, ribbons: false }),
    [items, colors, rules],
  );

  return <Legend entries={entries} symbolWidthPx={rules.legendSymbolPx} />;
}

export interface ArgLegendProps {
  items: readonly LegendItem[];
  colors: DrawingColors;
  rules: DrawingRules;
}
