import { useMemo } from "react";

import { Legend, lineSymbol } from "../drawing/Legend";
import type { SegmentLabels } from "../drawing/tooltip";

const RETICULATION_SYMBOL = (
  <>
    <path d="M0 2 C12 2 10 10 20 10" className="stroke-ink-muted fill-none" strokeWidth={1.5} strokeDasharray="4 3" />
    <circle cx={20} cy={10} r={3} className="fill-ground stroke-signal" strokeWidth={1.5} />
  </>
);

export function ArgLegend({ segments: [a, b] }: { segments: SegmentLabels }) {
  const entries = useMemo(
    () => [
      { label: `Segment ${a}`, symbol: lineSymbol("stroke-segment-a") },
      { label: `Segment ${b}`, symbol: lineSymbol("stroke-segment-b") },
      { label: "Both segments", symbol: lineSymbol("stroke-ink") },
      { label: "Reassortment", symbol: RETICULATION_SYMBOL },
    ],
    [a, b],
  );

  return <Legend entries={entries} />;
}
