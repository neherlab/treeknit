import {
  ADDED_SYMBOL,
  IMPUTED_SYMBOL,
  Legend,
  type LegendEntry,
  REASSORTMENT_SYMBOL,
  RIBBON_SYMBOL,
} from "../drawing/Legend";

const ENTRIES: LegendEntry[] = [
  { label: "Reassortment branch", symbol: REASSORTMENT_SYMBOL },
  { label: "Node added by resolution", symbol: ADDED_SYMBOL },
  { label: "Imputed leaf", symbol: IMPUTED_SYMBOL },
  { label: "Leaves of one MCC", symbol: RIBBON_SYMBOL },
];

export function TanglegramLegend() {
  return <Legend entries={ENTRIES} />;
}
