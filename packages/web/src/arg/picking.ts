import type { ArgView } from "@neherlab/treeknit-wasm";

import type { RowRange } from "../canvas/viewState";
import type { PickRules } from "../drawing/picking";
import { argClickSelection, argSelectionRows, type ArgTarget } from "../drawing/selection";
import { argTooltip } from "../drawing/tooltip";
import { nodeRows } from "../drawing/trees";
import { type ArgGeometry, argTargetAt } from "./geometry";

export const ARG_PICK_RULES: PickRules<ArgView, ArgGeometry, ArgTarget> = {
  targetAt: argTargetAt,
  tooltip: argTooltip,
  clickSelection: argClickSelection,
  targetRows: argTargetRows,
  selectionRows: argSelectionRows,
};

export function argTargetRows(view: ArgView, target: ArgTarget): RowRange | null {
  const node = target.kind === "edge" ? view.edges[target.edge]?.child : target.node;

  return node === undefined ? null : nodeRows(view.nodes[node]);
}
