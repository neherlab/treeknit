import type { ArgView, LabelMode } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { TreeCanvas } from "../canvas/TreeCanvas";
import type { TreeViewHandle } from "../canvas/useTreeView";
import { drawingCursor } from "../drawing/cursor";
import { useDrawingPicking } from "../drawing/picking";
import { argEmphasis, NO_SELECTION, type Selection } from "../drawing/selection";
import type { SegmentLabels } from "../drawing/tooltip";
import { argLeafLabels } from "../drawing/trees";
import { useDrawingCanvas, useDrawingGeometry } from "../drawing/useDrawingCanvas";
import { ArgLegend } from "./ArgLegend";
import { argColumn, argCurves, argFrame } from "./geometry";
import { argLayers, type ArgStyle } from "./layers";
import { argPickRules } from "./picking";

const ARG_GEOMETRY = { frame: argFrame, curves: argCurves };

export default function ArgCanvas({ data, view, labels, selection, onSelect, segments, label }: ArgCanvasProps) {
  const texts = useMemo(() => argLeafLabels(data), [data]);
  const canvas = useDrawingCanvas(view, labels, texts);
  const { colors, rules, tree, crossPx, labelsShown, leafLabels } = canvas;

  const column = useMemo(() => argColumn(crossPx, leafLabels.longestPx, rules), [crossPx, leafLabels.longestPx, rules]);

  const geometry = useDrawingGeometry(ARG_GEOMETRY, data, column, canvas);

  const style = useMemo<ArgStyle>(
    () => ({
      colors,
      rules,
      emphasis: argEmphasis(data, selection),
      labels: labelsShown,
      fontReady: leafLabels.fontReady,
    }),
    [colors, rules, data, selection, labelsShown, leafLabels.fontReady],
  );

  const layers = useMemo(() => argLayers(geometry, style), [geometry, style]);

  const minimapLayers = useMemo(
    () =>
      argLayers(geometry, {
        colors,
        rules,
        emphasis: argEmphasis(data, NO_SELECTION),
        labels: false,
        fontReady: false,
      }),
    [geometry, colors, rules, data],
  );

  const pickRules = useMemo(() => argPickRules(segments), [segments]);
  const picking = useDrawingPicking(pickRules, data, geometry, selection, onSelect);

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
      <ArgLegend items={data.legend} colors={colors} rules={rules} />
    </TreeCanvas>
  );
}

export interface ArgCanvasProps {
  data: ArgView;
  view: TreeViewHandle;
  labels: LabelMode;
  selection: Selection;
  onSelect: (selection: Selection) => void;
  segments: SegmentLabels;
  label: string;
}
