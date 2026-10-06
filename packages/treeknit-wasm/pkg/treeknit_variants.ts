import type { LabelMode, Scale, TreeVersion } from "./treeknit_wasm";

export const TREE_VERSION_VALUES = ["input", "resolved", "imputed"] as const satisfies readonly TreeVersion[];

export const SCALE_VALUES = ["div", "depth"] as const satisfies readonly Scale[];

export const LABEL_MODE_VALUES = ["auto", "on", "off"] as const satisfies readonly LabelMode[];
