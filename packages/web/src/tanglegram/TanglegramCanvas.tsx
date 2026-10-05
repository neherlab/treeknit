import type { PickingInfo } from "@deck.gl/core";
import type { LabelMode, PairView } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";

import { useDrawingRules } from "../analysis/queries";
import { useDrawingColors } from "../canvas/drawingColors";
import { labelCharacters, labelsVisible, useLabelFontReady } from "../canvas/labels";
import { useFadeIn } from "../canvas/motion";
import { curveRowPx } from "../canvas/projection";
import { TreeCanvas } from "../canvas/TreeCanvas";
import type { TreeView } from "../canvas/useTreeView";
import { crossExtent, type RowRange } from "../canvas/viewState";
import { drawingCursor } from "../drawing/cursor";
import { canvasTextMeasure, longestLabelPx } from "../drawing/labelWidth";
import {
  pairClickSelection,
  pairEmphasis,
  pairSelectionRows,
  type PairTarget,
  type Selection,
} from "../drawing/selection";
import { pairTooltip } from "../drawing/tooltip";
import { tooltipContent } from "../drawing/tooltipContent";
import { leafNames, leafRows, pairLeafRows, rowSpan } from "../drawing/trees";
import { labelColumnPx, tanglegramColumns } from "./columns";
import { pairTargetAt, tanglegramGeometry } from "./geometry";
import { ribbonsShown, tanglegramLayers } from "./layers";
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
  const labelText = useMemo(() => labelCharacters([...leafNames(data.left), ...leafNames(data.right)]), [data]);
  const fontReady = useLabelFontReady(labelText);
  const fade = useFadeIn(resultKey);
  const { frame, rowPx } = view;
  const leafAxis = frame?.leafAxis ?? "y";
  const crossPx = frame === undefined ? 0 : crossExtent(frame);
  const labelled = leafAxis === "y" && labels !== "off";

  const longestLabel = useMemo(() => {
    if (!labelled || !fontReady) {
      return 0;
    }

    const measure = canvasTextMeasure();

    return Math.max(
      longestLabelPx(leafNames(data.left), measure, rules.labelMaxChars),
      longestLabelPx(leafNames(data.right), measure, rules.labelMaxChars),
    );
  }, [data, labelled, fontReady, rules]);

  const columns = useMemo(
    () => tanglegramColumns(crossPx, labelColumnPx(crossPx, longestLabel)),
    [crossPx, longestLabel],
  );

  const sampledRowPx = curveRowPx(rowPx);

  const geometry = useMemo(
    () => tanglegramGeometry(data, columns, leafAxis, sampledRowPx),
    [data, columns, leafAxis, sampledRowPx],
  );

  const emphasis = useMemo(() => pairEmphasis(data, selection), [data, selection]);
  const ribbons = ribbonsShown(rowPx, rules);
  const labelsShown = leafAxis === "y" && labelsVisible(labels, rowPx, rules);

  const layers = useMemo(
    () =>
      tanglegramLayers(geometry, {
        colors,
        colorByMcc,
        emphasis,
        ribbons,
        labels: labelsShown,
        fontReady,
        labelMaxChars: rules.labelMaxChars,
        fade,
      }),
    [geometry, colors, colorByMcc, emphasis, ribbons, labelsShown, fontReady, rules, fade],
  );

  const minimapLayers = useMemo(
    () =>
      tanglegramLayers(geometry, {
        colors,
        colorByMcc,
        emphasis: pairEmphasis(data, {}),
        ribbons: true,
        labels: false,
        fontReady: false,
        labelMaxChars: rules.labelMaxChars,
        fade: 1,
      }),
    [geometry, colors, colorByMcc, data, rules],
  );

  const targetOf = useCallback(
    (info: PickingInfo): PairTarget | undefined =>
      info.layer === null || info.layer === undefined ? undefined : pairTargetAt(geometry, info.layer.id, info.index),
    [geometry],
  );

  const getTooltip = useCallback(
    (info: PickingInfo) => {
      const target = targetOf(info);

      return target === undefined ? null : tooltipContent(pairTooltip(data, target));
    },
    [data, targetOf],
  );

  const onClick = useCallback(
    (info: PickingInfo) => {
      onSelect(pairClickSelection(data, targetOf(info)));
    },
    [data, onSelect, targetOf],
  );

  const onCladeZoom = useCallback(
    (info: PickingInfo): RowRange | null => {
      const target = targetOf(info);

      return target === undefined ? null : targetRows(data, target);
    },
    [data, targetOf],
  );

  const onSelectionZoom = useCallback(() => pairSelectionRows(data, selection), [data, selection]);

  return (
    <TreeCanvas
      view={view}
      label={label}
      layers={layers}
      minimapLayers={minimapLayers}
      getTooltip={getTooltip}
      onClick={onClick}
      onCladeZoom={onCladeZoom}
      onSelectionZoom={onSelectionZoom}
      getCursor={drawingCursor}
      className="h-full"
    >
      <TanglegramLegend />
    </TreeCanvas>
  );
}

export interface TanglegramCanvasProps {
  data: PairView;
  view: TreeView;
  labels: LabelMode;
  colorByMcc: boolean;
  selection: Selection;
  onSelect: (selection: Selection) => void;
  resultKey: string;
  label: string;
}

function targetRows(data: PairView, target: PairTarget): RowRange | null {
  if (target.kind === "ribbon") {
    const block = data.blocks[target.block];

    return block === undefined ? null : rowSpan([...block.left, ...block.right]);
  }

  if (target.kind === "link") {
    const link = data.links[target.link];
    const leaf = link === undefined ? undefined : data.left.nodes[link.left];

    return leaf === undefined ? null : pairLeafRows(data.left, data.right, leaf.name);
  }

  return leafRows(data[target.side].nodes, target.node);
}
