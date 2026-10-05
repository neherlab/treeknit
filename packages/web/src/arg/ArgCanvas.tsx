import type { ArgView, LabelMode } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";

import { useDrawingRules } from "../analysis/queries";
import { useDrawingColors } from "../canvas/drawingColors";
import { labelsVisible } from "../canvas/labels";
import { curveRowPx } from "../canvas/projection";
import { TreeCanvas } from "../canvas/TreeCanvas";
import type { TreeView } from "../canvas/useTreeView";
import { crossExtent } from "../canvas/viewState";
import { drawingCursor } from "../drawing/cursor";
import { labelColumnPx, useLeafLabels } from "../drawing/labelWidth";
import { useDrawingPicking } from "../drawing/picking";
import { argEmphasis, NO_SELECTION, type Selection } from "../drawing/selection";
import { innerWidthPx } from "../drawing/spacing";
import type { SegmentLabels } from "../drawing/tooltip";
import { argLeafNames } from "../drawing/trees";
import { ArgLegend } from "./ArgLegend";
import { argColumn, argGeometry } from "./geometry";
import { argLayers, type ArgStyle } from "./layers";
import { argPickRules } from "./picking";

export default function ArgCanvas({ data, view, labels, selection, onSelect, segments, label }: ArgCanvasProps) {
  const colors = useDrawingColors();
  const rules = useDrawingRules();
  const { frame, rowPx } = view;
  const leafAxis = frame?.leafAxis ?? "y";
  const crossPx = frame === undefined ? 0 : crossExtent(frame);
  const names = useMemo(() => argLeafNames(data), [data]);
  const labelsShown = leafAxis === "y" && labelsVisible(labels, rowPx, rules);
  const leafLabels = useLeafLabels(names, labelsShown, rules.labelMaxChars);

  const column = useMemo(
    () => argColumn(crossPx, labelColumnPx(innerWidthPx(crossPx), leafLabels.longestPx)),
    [crossPx, leafLabels.longestPx],
  );

  const sampledRowPx = curveRowPx(rowPx);

  const geometry = useMemo(
    () => argGeometry(data, column, leafAxis, sampledRowPx),
    [data, column, leafAxis, sampledRowPx],
  );

  const style = useMemo<ArgStyle>(
    () => ({
      colors,
      emphasis: argEmphasis(data, selection),
      labels: labelsShown,
      fontReady: leafLabels.fontReady,
      labelMaxChars: rules.labelMaxChars,
    }),
    [colors, data, selection, labelsShown, rules, leafLabels.fontReady],
  );

  const layers = useMemo(() => argLayers(geometry, style), [geometry, style]);

  const minimapLayers = useMemo(
    () =>
      argLayers(geometry, {
        colors,
        emphasis: argEmphasis(data, NO_SELECTION),
        labels: false,
        fontReady: false,
        labelMaxChars: rules.labelMaxChars,
      }),
    [geometry, colors, data, rules],
  );

  const pickRules = useMemo(() => argPickRules(segments), [segments]);
  const picking = useDrawingPicking(pickRules, data, geometry, selection, onSelect);

  return (
    <TreeCanvas
      view={view}
      label={label}
      layers={layers}
      minimapLayers={minimapLayers}
      getCursor={drawingCursor}
      className="h-full"
      {...picking}
    >
      <ArgLegend colors={colors} segments={segments} />
    </TreeCanvas>
  );
}

export interface ArgCanvasProps {
  data: ArgView;
  view: TreeView;
  labels: LabelMode;
  selection: Selection;
  onSelect: (selection: Selection) => void;
  segments: SegmentLabels;
  label: string;
}
