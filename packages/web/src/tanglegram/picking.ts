import type { PairView } from "@neherlab/treeknit-wasm";

import type { RowRange } from "../canvas/viewState";
import type { PickRules } from "../drawing/picking";
import { pairClickSelection, pairSelectionRows, type PairTarget } from "../drawing/selection";
import { pairTooltip } from "../drawing/tooltip";
import { leafRows, pairLeafRows, rowSpan } from "../drawing/trees";
import { pairTargetAt, type TanglegramGeometry } from "./geometry";

export const PAIR_PICK_RULES: PickRules<PairView, TanglegramGeometry, PairTarget> = {
  targetAt: pairTargetAt,
  tooltip: pairTooltip,
  clickSelection: pairClickSelection,
  targetRows: pairTargetRows,
  selectionRows: pairSelectionRows,
};

export function pairTargetRows(view: PairView, target: PairTarget): RowRange | null {
  if (target.kind === "ribbon") {
    const block = view.blocks[target.block];

    return block === undefined ? null : rowSpan([...block.left, ...block.right]);
  }

  if (target.kind === "link") {
    const link = view.links[target.link];
    const leaf = link === undefined ? undefined : view.left.nodes[link.left];

    return leaf === undefined ? null : pairLeafRows(view.left, view.right, leaf.name);
  }

  return leafRows(view[target.side].nodes, target.node);
}
