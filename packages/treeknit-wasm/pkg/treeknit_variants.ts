import type { LabelMode, LegendKind, Scale, TreeVersion } from "./treeknit_wasm";

export const TREE_VERSION_VALUES = ["input", "resolved", "imputed"] as const satisfies readonly TreeVersion[];

export const SCALE_VALUES = ["div", "depth"] as const satisfies readonly Scale[];

export const LABEL_MODE_VALUES = ["auto", "on", "off"] as const satisfies readonly LabelMode[];

export const LEGEND_LABELS = {
  "reassortmentBranch": "Reassortment branch",
  "addedNode": "Node added by resolution or imputation",
  "imputedLeaf": "Imputed leaf",
  "links": "Leaves of one MCC",
  "noMcc": "No MCC",
  "segmentA": "Segment",
  "segmentB": "Segment",
  "bothSegments": "Both segments",
  "reassortment": "Reassortment",
} as const satisfies Record<LegendKind, string>;
