import { useMemo } from "react";

import { Legend } from "../drawing/Legend";
import { type LegendStyle, tanglegramLegend } from "./legend";

export function TanglegramLegend({ colors, colorByMcc, ribbons }: LegendStyle) {
  const entries = useMemo(() => tanglegramLegend({ colors, colorByMcc, ribbons }), [colors, colorByMcc, ribbons]);

  return <Legend entries={entries} />;
}
