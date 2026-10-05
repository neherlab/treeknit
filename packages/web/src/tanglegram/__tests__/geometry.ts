import type { PairView } from "@neherlab/treeknit-wasm";

import type { LeafAxis } from "../../canvas/viewState";
import type { TanglegramColumns } from "../columns";
import { tanglegramCurves, tanglegramFrame, type TanglegramGeometry } from "../geometry";

export function tanglegramGeometry(
  view: PairView,
  columns: TanglegramColumns,
  leafAxis: LeafAxis,
  curveRowPx: number,
): TanglegramGeometry {
  return { ...tanglegramFrame(view, columns, leafAxis), ...tanglegramCurves(view, columns, leafAxis, curveRowPx) };
}
