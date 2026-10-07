import type { PairView } from "@neherlab/treeknit-wasm";
import { match } from "ts-pattern";

import type { RowRange } from "../canvas/viewState";
import type { PickRules } from "../drawing/picking";
import { pairClickSelection, pairSelectionRows, type PairTarget } from "../drawing/selection";
import { pairTooltip } from "../drawing/tooltip";
import { nodeRows, rowSpan } from "../drawing/trees";
import { pairTargetAt, type TanglegramGeometry } from "./geometry";

export const PAIR_PICK_RULES: PickRules<PairView, TanglegramGeometry, PairTarget> = {
  targetAt: pairTargetAt,
  tooltip: pairTooltip,
  clickSelection: pairClickSelection,
  targetRows: pairTargetRows,
  selectionRows: pairSelectionRows,
};

export function pairTargetRows(view: PairView, target: PairTarget): RowRange | null {
  return match(target)
    .with({ kind: "ribbon" }, ({ block }) => {
      const shown = view.blocks[block];

      return shown === undefined ? null : rowSpan([...shown.left, ...shown.right]);
    })
    .with({ kind: "link" }, ({ link }) => {
      const shown = view.links[link];

      return shown === undefined ? null : nodeRows(view.left.nodes[shown.left], view.right.nodes[shown.right]);
    })
    .with({ kind: "node" }, ({ side, node }) => nodeRows(view[side].nodes[node]))
    .exhaustive();
}
