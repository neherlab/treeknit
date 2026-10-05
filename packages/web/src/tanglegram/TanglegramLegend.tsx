import { useMemo } from "react";

import { Legend } from "../drawing/Legend";
import { type LegendStyle, tanglegramLegend } from "./legend";

export function TanglegramLegend({ colors, rules, colorByMcc, ribbons }: LegendStyle) {
  const entries = useMemo(
    () => tanglegramLegend({ colors, rules, colorByMcc, ribbons }),
    [colors, rules, colorByMcc, ribbons],
  );

  return <Legend entries={entries} />;
}
