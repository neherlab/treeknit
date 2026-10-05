import type { LabelMode, PairView } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useDrawingRules } from "../analysis/queries";
import { useDrawingColors } from "../canvas/drawingColors";
import { labelsVisible } from "../canvas/labels";
import { useFadeIn } from "../canvas/motion";
import { curveRowPx } from "../canvas/projection";
import { TreeCanvas } from "../canvas/TreeCanvas";
import { type TreeViewHandle, useTreeView } from "../canvas/useTreeView";
import { crossExtent } from "../canvas/viewState";
import { drawingCursor } from "../drawing/cursor";
import { labelColumnPx, useLeafLabels } from "../drawing/labelWidth";
import { useDrawingPicking } from "../drawing/picking";
import { NO_SELECTION, pairEmphasis, type Selection } from "../drawing/selection";
import { innerWidthPx } from "../drawing/spacing";
import { pairLeafNames } from "../drawing/trees";
import { tanglegramColumns } from "./columns";
import { tanglegramCurves, tanglegramFrame } from "./geometry";
import { type PairStyle, ribbonsShown, tanglegramLayers } from "./layers";
import { PAIR_PICK_RULES } from "./picking";
import { TanglegramLegend } from "./TanglegramLegend";

export default function TanglegramCanvas({
  data,
  view,
  labels,
  colorByMcc,
  selection,
  onSelect,
  resultKey,
  label,
}: TanglegramCanvasProps) {
  const colors = useDrawingColors();
  const rules = useDrawingRules();
  const fade = useFadeIn(resultKey);
  const tree = useTreeView(view);
  const { frame, rowPx } = tree;
  const leafAxis = frame?.leafAxis ?? "y";
  const crossPx = frame === undefined ? 0 : crossExtent(frame);
  const names = useMemo(() => pairLeafNames(data), [data]);
  const labelsShown = leafAxis === "y" && labelsVisible(labels, rowPx, rules);
  const leafLabels = useLeafLabels(names, labelsShown, rules.labelMaxChars);

  const columns = useMemo(
    () => tanglegramColumns(crossPx, labelColumnPx(innerWidthPx(crossPx) / 2, leafLabels.longestPx)),
    [crossPx, leafLabels.longestPx],
  );

  const sampledRowPx = curveRowPx(rowPx);

  const frameGeometry = useMemo(() => tanglegramFrame(data, columns, leafAxis), [data, columns, leafAxis]);

  const curves = useMemo(
    () => tanglegramCurves(data, columns, leafAxis, sampledRowPx),
    [data, columns, leafAxis, sampledRowPx],
  );

  const geometry = useMemo(() => ({ ...frameGeometry, ...curves }), [frameGeometry, curves]);

  const style = useMemo<PairStyle>(
    () => ({
      colors,
      colorByMcc,
      emphasis: pairEmphasis(data, selection),
      ribbons: ribbonsShown(rowPx, rules),
      labels: labelsShown,
      fontReady: leafLabels.fontReady,
      labelMaxChars: rules.labelMaxChars,
      fade,
    }),
    [colors, colorByMcc, data, selection, rowPx, rules, labelsShown, leafLabels.fontReady, fade],
  );

  const layers = useMemo(() => tanglegramLayers(geometry, style), [geometry, style]);

  const minimapLayers = useMemo(
    () =>
      tanglegramLayers(geometry, {
        colors,
        colorByMcc,
        emphasis: pairEmphasis(data, NO_SELECTION),
        ribbons: true,
        labels: false,
        fontReady: false,
        labelMaxChars: rules.labelMaxChars,
        fade: 1,
      }),
    [geometry, colors, colorByMcc, data, rules],
  );

  const picking = useDrawingPicking(PAIR_PICK_RULES, data, geometry, selection, onSelect);

  return (
    <TreeCanvas
      view={tree}
      label={label}
      layers={layers}
      minimapLayers={minimapLayers}
      getCursor={drawingCursor}
      className="h-full"
      {...picking}
    >
      <TanglegramLegend colors={colors} colorByMcc={colorByMcc} ribbons={style.ribbons} />
    </TreeCanvas>
  );
}

export interface TanglegramCanvasProps {
  data: PairView;
  view: TreeViewHandle;
  labels: LabelMode;
  colorByMcc: boolean;
  selection: Selection;
  onSelect: (selection: Selection) => void;
  resultKey: string;
  label: string;
}
