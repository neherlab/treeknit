import type { LabelMode, LegendKind, Scale, TreeVersion } from "./treeknit_wasm";

export const TREE_VERSION_VALUES = ["input", "resolved", "imputed"] as const satisfies readonly TreeVersion[];

export const TREE_VERSION_DEFAULT = "resolved" as const satisfies TreeVersion;

export const SCALE_VALUES = ["div", "depth"] as const satisfies readonly Scale[];

export const SCALE_DEFAULT = "div" as const satisfies Scale;

export const LABEL_MODE_VALUES = ["auto", "on", "off"] as const satisfies readonly LabelMode[];

export const LABEL_MODE_DEFAULT = "auto" as const satisfies LabelMode;

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

export const MIN_TREES = 2;

export const ARG_TREE_COUNT = 2;
