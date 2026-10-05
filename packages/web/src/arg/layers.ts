import type { LayersList } from "@deck.gl/core";

import type { Rgba } from "../canvas/color";
import type { DrawingColors } from "../canvas/drawingColors";
import { labelLayer } from "../canvas/layers/labelLayer";
import { leaderLayer } from "../canvas/layers/leaderLayer";
import { branchLayer, ringLayer, selectionLayer } from "../canvas/layers/treeLayers";
import type { ArgEmphasis } from "../drawing/selection";
import { LABEL_GAP_PX } from "../drawing/spacing";
import { ARG_LAYER, type ArgGeometry } from "./geometry";

export interface ArgStyle {
  colors: DrawingColors;
  emphasis: ArgEmphasis;
  labels: boolean;
  fontReady: boolean;
}

export function argEdgeColor(
  segments: readonly number[],
  colors: Pick<DrawingColors, "segmentA" | "segmentB" | "ink">,
): Rgba {
  if (segments.length === 1 && segments[0] === 0) {
    return colors.segmentA;
  }

  return segments.length === 1 && segments[0] === 1 ? colors.segmentB : colors.ink;
}

export function argSelectionPositions(geometry: ArgGeometry, emphasis: ArgEmphasis) {
  return [emphasis.leaf, emphasis.node].flatMap((node) => {
    const position = node === undefined ? undefined : geometry.nodes[node];

    return position === undefined ? [] : [position];
  });
}

export function argLayers(geometry: ArgGeometry, style: ArgStyle): LayersList {
  const { colors } = style;
  const triggers = [colors];

  return [
    branchLayer({
      id: ARG_LAYER.edges,
      data: geometry.edges,
      getPath: (item) => item.path,
      getColor: (item) => argEdgeColor(item.segments, colors),
      colors,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: ARG_LAYER.reticulations,
      data: geometry.reticulations,
      getPath: (item) => item.path,
      getColor: (item) => argEdgeColor(item.segments, colors),
      colors,
      dashed: true,
      colorTriggers: triggers,
    }),
    leaderLayer({
      id: ARG_LAYER.leaders,
      data: geometry.leaders,
      colors,
      visible: style.labels && style.fontReady,
    }),
    ringLayer({
      id: ARG_LAYER.hybrids,
      data: geometry.hybrids,
      getPosition: (item) => item.position,
      getLineColor: colors.signal,
      colors,
      colorTriggers: triggers,
    }),
    labelLayer({
      id: ARG_LAYER.labels,
      data: geometry.labels,
      fontReady: style.fontReady,
      visible: style.labels,
      getPosition: (item) => item.position,
      getText: (item) => item.text,
      getColor: colors.ink,
      anchor: "start",
      offsetPx: [LABEL_GAP_PX, 0],
      pickable: true,
      colorTriggers: triggers,
    }),
    selectionLayer("arg-selection", argSelectionPositions(geometry, style.emphasis), colors),
  ];
}
