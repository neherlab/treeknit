import type { ArgView } from "@neherlab/treeknit-wasm";

import type { Column } from "../../canvas/projection";
import type { LeafAxis } from "../../canvas/viewState";
import { argCurves, argFrame, type ArgGeometry } from "../geometry";

export function argGeometry(view: ArgView, column: Column, leafAxis: LeafAxis, curveRowPx: number): ArgGeometry {
  return { ...argFrame(view, column, leafAxis), ...argCurves(view, column, leafAxis, curveRowPx) };
}
