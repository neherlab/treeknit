import type { LabelMode, PairView } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useFadeIn } from "../canvas/motion";
import { TreeCanvas } from "../canvas/TreeCanvas";
import type { TreeViewHandle } from "../canvas/useTreeView";
import { drawingCursor } from "../drawing/cursor";
import { labelColumnPx } from "../drawing/labelWidth";
import { useDrawingPicking } from "../drawing/picking";
import { NO_SELECTION, pairEmphasis, type Selection } from "../drawing/selection";
import { innerWidthPx } from "../drawing/spacing";
import { pairLeafLabels } from "../drawing/trees";
import { useDrawingCanvas, useDrawingGeometry } from "../drawing/useDrawingCanvas";
import { tanglegramColumns } from "./columns";
import { tanglegramCurves, tanglegramFrame } from "./geometry";
import { type PairStyle, ribbonsShown, tanglegramLayers } from "./layers";
import { PAIR_PICK_RULES } from "./picking";
import { TanglegramLegend } from "./TanglegramLegend";

const PAIR_GEOMETRY = { frame: tanglegramFrame, curves: tanglegramCurves };

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
  const fade = useFadeIn(resultKey);
  const texts = useMemo(() => pairLeafLabels(data), [data]);
  const canvas = useDrawingCanvas(view, labels, texts);
  const { colors, rules, tree, crossPx, labelsShown, leafLabels } = canvas;
  const { rowPx } = tree;

  const columns = useMemo(
    () => tanglegramColumns(crossPx, labelColumnPx(innerWidthPx(crossPx) / 2, leafLabels.longestPx)),
    [crossPx, leafLabels.longestPx],
  );

  const geometry = useDrawingGeometry(PAIR_GEOMETRY, data, columns, canvas);

  const style = useMemo<PairStyle>(
    () => ({
      colors,
      colorByMcc,
      emphasis: pairEmphasis(data, selection),
      ribbons: ribbonsShown(rowPx, rules),
      labels: labelsShown,
      fontReady: leafLabels.fontReady,
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
        fade: 1,
      }),
    [geometry, colors, colorByMcc, data],
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
