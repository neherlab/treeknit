import type { LegendItem } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { Legend } from "../drawing/Legend";
import { legendEntries, type LegendStyle } from "../drawing/legendMarks";

export function TanglegramLegend({ items, colors, rules, colorByMcc, ribbons }: TanglegramLegendProps) {
  const entries = useMemo(
    () => legendEntries(items, { colors, rules, colorByMcc, ribbons }),
    [items, colors, rules, colorByMcc, ribbons],
  );

  return <Legend entries={entries} symbolWidthPx={rules.legendSymbolPx} />;
}

export interface TanglegramLegendProps extends LegendStyle {
  items: readonly LegendItem[];
}
