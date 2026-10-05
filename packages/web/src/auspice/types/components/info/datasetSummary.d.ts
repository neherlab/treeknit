import type { AuspiceNode } from "auspice/src/state";
import type { TFunction } from "i18next";

export declare function datasetSummary(params: {
  nodes: readonly AuspiceNode[] | null;
  visibility: readonly number[] | null;
  mainTreeNumTips: number | undefined;
  branchLengthsToDisplay: string;
  t: TFunction;
}): string;
