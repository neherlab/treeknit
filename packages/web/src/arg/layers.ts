import type { LayersList } from "@deck.gl/core";
import type { DrawingRules } from "@neherlab/treeknit-wasm";

import { type DrawingColors, roleColor } from "../canvas/drawingColors";
import { labelLayer } from "../canvas/layers/labelLayer";
import { leaderLayer } from "../canvas/layers/leaderLayer";
import { branchLayer, ringLayer, selectionLayer } from "../canvas/layers/treeLayers";
import type { ArgEmphasis } from "../drawing/selection";
import { ARG_LAYER, type ArgGeometry } from "./geometry";

export interface ArgStyle {
  colors: DrawingColors;
  rules: DrawingRules;
  emphasis: ArgEmphasis;
  labels: boolean;
  fontReady: boolean;
}

export function argSelectionPositions(geometry: ArgGeometry, emphasis: ArgEmphasis) {
  return [emphasis.leaf, emphasis.node].flatMap((node) => {
    const position = node === undefined ? undefined : geometry.nodes[node];

    return position === undefined ? [] : [position];
  });
}

export function argLayers(geometry: ArgGeometry, style: ArgStyle): LayersList {
  const { colors, rules } = style;
  const triggers = [colors];

  return [
    branchLayer({
      id: ARG_LAYER.edges,
      data: geometry.edges,
      getPath: (item) => item.path,
      getColor: (item) => roleColor(colors, item.color, null),
      colors,
      widthPx: rules.branchWidthPx,
      colorTriggers: triggers,
    }),
    branchLayer({
      id: ARG_LAYER.reticulations,
      data: geometry.reticulations,
      getPath: (item) => item.path,
      getColor: (item) => roleColor(colors, item.color, null),
      colors,
      widthPx: rules.branchWidthPx,
      dashPx: rules.dashPx,
      colorTriggers: triggers,
    }),
    leaderLayer({
      id: ARG_LAYER.leaders,
      data: geometry.leaders,
      colors,
      rules,
      visible: style.labels && style.fontReady,
    }),
    ringLayer({
      id: ARG_LAYER.hybrids,
      data: geometry.hybrids,
      getPosition: (item) => item.position,
      getLineColor: colors.signal,
      colors,
      rules,
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
      offsetPx: [rules.labelGapPx, 0],
      pickable: true,
      colorTriggers: triggers,
    }),
    selectionLayer("arg-selection", argSelectionPositions(geometry, style.emphasis), colors),
  ];
}
