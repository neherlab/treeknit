import { datasetSummary } from "auspice/src/components/info/datasetSummary";
import type { TFunction } from "i18next";

import type { AuspiceControlsState, AuspiceState } from "./state";

export function auspiceSummary(
  { tree, metadata, controls }: Pick<AuspiceState, "tree" | "metadata" | "controls">,
  t: TFunction,
): string {
  return datasetSummary({
    nodes: tree.nodes,
    visibility: tree.visibility,
    mainTreeNumTips: metadata.mainTreeNumTips,
    temporalTree: isTemporalTree(controls),
    t,
  });
}

export function isTemporalTree({
  availableDistanceMeasures,
}: Pick<AuspiceControlsState, "availableDistanceMeasures">): boolean {
  return availableDistanceMeasures.includes("num_date");
}
