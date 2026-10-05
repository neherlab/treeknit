import type { DrawingRules, LabelMode } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useDrawingRules } from "../analysis/queries";
import { type DrawingColors, useDrawingColors } from "../canvas/drawingColors";
import { labelsVisible } from "../canvas/labels";
import { curveRowPx } from "../canvas/projection";
import { type TreeView, type TreeViewHandle, useTreeView } from "../canvas/useTreeView";
import { crossExtent, type LeafAxis } from "../canvas/viewState";
import { type LeafLabels, useLeafLabels } from "./labelWidth";

export interface DrawingCanvas {
  colors: DrawingColors;
  rules: DrawingRules;
  tree: TreeView;
  leafAxis: LeafAxis;
  crossPx: number;
  labelsShown: boolean;
  leafLabels: LeafLabels;
  curveRowPx: number;
}

export interface GeometryBuilder<D, L, F, C> {
  frame(data: D, layout: L, leafAxis: LeafAxis): F;
  curves(data: D, layout: L, leafAxis: LeafAxis, curveRowPx: number): C;
}

export function useDrawingCanvas(
  view: TreeViewHandle,
  labels: LabelMode,
  labelTexts: readonly string[],
): DrawingCanvas {
  const colors = useDrawingColors();
  const rules = useDrawingRules();
  const tree = useTreeView(view);
  const { frame, rowPx } = tree;
  const leafAxis = frame?.leafAxis ?? "y";
  const labelsShown = leafAxis === "y" && labelsVisible(labels, rowPx, rules);
  const leafLabels = useLeafLabels(labelTexts, labelsShown);

  return {
    colors,
    rules,
    tree,
    leafAxis,
    crossPx: frame === undefined ? 0 : crossExtent(frame),
    labelsShown,
    leafLabels,
    curveRowPx: curveRowPx(rowPx),
  };
}

export function useDrawingGeometry<D, L, F extends object, C extends object>(
  builder: GeometryBuilder<D, L, F, C>,
  data: D,
  layout: L,
  canvas: Pick<DrawingCanvas, "leafAxis" | "curveRowPx">,
): F & C {
  const { leafAxis, curveRowPx: rowPx } = canvas;
  const frame = useMemo(() => builder.frame(data, layout, leafAxis), [builder, data, layout, leafAxis]);
  const curves = useMemo(() => builder.curves(data, layout, leafAxis, rowPx), [builder, data, layout, leafAxis, rowPx]);

  return useMemo(() => ({ ...frame, ...curves }), [frame, curves]);
}
