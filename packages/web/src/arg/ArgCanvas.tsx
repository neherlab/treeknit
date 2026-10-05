import type { PickingInfo } from "@deck.gl/core";
import type { ArgView, LabelMode } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";

import { useDrawingColors } from "../canvas/drawingColors";
import { labelsVisible, useLabelFontReady } from "../canvas/labels";
import { curveRowPx } from "../canvas/projection";
import { TreeCanvas } from "../canvas/TreeCanvas";
import type { TreeView } from "../canvas/useTreeView";
import { crossExtent, type RowRange } from "../canvas/viewState";
import { drawingCursor } from "../drawing/cursor";
import { canvasTextMeasure, longestLabelPx } from "../drawing/labelWidth";
import { argClickSelection, argEmphasis, type ArgTarget, type Selection } from "../drawing/selection";
import { argTooltip, type SegmentLabels } from "../drawing/tooltip";
import { tooltipContent } from "../drawing/tooltipContent";
import { leafRows } from "../drawing/trees";
import { ArgLegend } from "./ArgLegend";
import { argColumn, argGeometry, argTargetAt } from "./geometry";
import { argLayers } from "./layers";

export default function ArgCanvas({ data, view, labels, selection, onSelect, segments, label }: ArgCanvasProps) {
  const colors = useDrawingColors();
  const fontReady = useLabelFontReady();
  const { frame, rowPx } = view;
  const leafAxis = frame?.leafAxis ?? "y";
  const crossPx = frame === undefined ? 0 : crossExtent(frame);
  const labelled = leafAxis === "y" && labels !== "off";

  const longestLabel = useMemo(() => {
    if (!labelled || !fontReady) {
      return 0;
    }

    const names = data.nodes.flatMap((node) => (node.leaf ? [node.label] : []));

    return longestLabelPx(names, canvasTextMeasure());
  }, [data, labelled, fontReady]);

  const column = useMemo(() => argColumn(crossPx, longestLabel), [crossPx, longestLabel]);
  const sampledRowPx = curveRowPx(rowPx);

  const geometry = useMemo(
    () => argGeometry(data, column, leafAxis, sampledRowPx),
    [data, column, leafAxis, sampledRowPx],
  );

  const emphasis = useMemo(() => argEmphasis(data, selection), [data, selection]);
  const labelsShown = leafAxis === "y" && labelsVisible(labels, rowPx);

  const layers = useMemo(
    () => argLayers(geometry, { colors, emphasis, labels: labelsShown, fontReady }),
    [geometry, colors, emphasis, labelsShown, fontReady],
  );

  const minimapLayers = useMemo(
    () => argLayers(geometry, { colors, emphasis: argEmphasis(data, {}), labels: false, fontReady: false }),
    [geometry, colors, data],
  );

  const targetOf = useCallback(
    (info: PickingInfo): ArgTarget | undefined =>
      info.layer === null || info.layer === undefined ? undefined : argTargetAt(geometry, info.layer.id, info.index),
    [geometry],
  );

  const getTooltip = useCallback(
    (info: PickingInfo) => {
      const target = targetOf(info);

      return target === undefined ? null : tooltipContent(argTooltip(data, target, segments));
    },
    [data, segments, targetOf],
  );

  const onClick = useCallback(
    (info: PickingInfo) => {
      onSelect(argClickSelection(data, targetOf(info)));
    },
    [data, onSelect, targetOf],
  );

  const onCladeZoom = useCallback(
    (info: PickingInfo): RowRange | null => {
      const target = targetOf(info);
      const node = target?.kind === "edge" ? data.edges[target.edge]?.child : target?.node;

      return node === undefined ? null : leafRows(data.nodes, node);
    },
    [data, targetOf],
  );

  return (
    <TreeCanvas
      view={view}
      label={label}
      layers={layers}
      minimapLayers={minimapLayers}
      getTooltip={getTooltip}
      onClick={onClick}
      onCladeZoom={onCladeZoom}
      getCursor={drawingCursor}
      className="h-full"
    >
      <ArgLegend segments={segments} />
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
