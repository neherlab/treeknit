import type { Accessor, Color } from "@deck.gl/core";
import type { DrawingRules } from "@neherlab/treeknit-wasm";

import { type Rgba, withOpacity } from "../color";
import type { DrawingColors } from "../drawingColors";
import type { WorldPosition } from "../projection";
import { markLayer } from "./markLayer";
import { pathLayer } from "./pathLayer";

const SELECTION_RADIUS_PX = 7;

const SELECTION_LINE_PX = 2;

const HOVER_OPACITY = 0.35;

export interface BranchLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPath: (object: D) => WorldPosition[];
  getColor: Accessor<D, Color>;
  colors: DrawingColors;
  widthPx: number;
  dashPx?: readonly [number, number];
  opacity?: number;
  visible?: boolean;
  pickable?: boolean;
  colorTriggers: readonly unknown[];
}

export function branchLayer<D>({
  id,
  data,
  getPath,
  getColor,
  colors,
  widthPx,
  dashPx,
  opacity = 1,
  visible = true,
  pickable = true,
  colorTriggers,
}: BranchLayerOptions<D>) {
  return pathLayer({
    id,
    data,
    getPath,
    getColor,
    widthPx,
    ...(dashPx === undefined ? undefined : { dashPx }),
    opacity,
    visible,
    pickable,
    highlightColor: hoverColor(colors),
    colorTriggers,
  });
}

export interface RingLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPosition: (object: D) => WorldPosition;
  getLineColor: Accessor<D, Color>;
  colors: DrawingColors;
  rules: DrawingRules;
  colorTriggers: readonly unknown[];
}

export function ringLayer<D>({
  id,
  data,
  getPosition,
  getLineColor,
  colors,
  rules,
  colorTriggers,
}: RingLayerOptions<D>) {
  return markLayer({
    id,
    data,
    getPosition,
    getLineColor,
    getFillColor: colors.ground,
    radiusPx: rules.markRadiusPx,
    lineWidthPx: rules.markLinePx,
    colorTriggers,
  });
}

export function selectionLayer(id: string, positions: readonly WorldPosition[], colors: DrawingColors) {
  return markLayer({
    id,
    data: positions,
    getPosition: (position) => position,
    getLineColor: colors.focus,
    radiusPx: SELECTION_RADIUS_PX,
    lineWidthPx: SELECTION_LINE_PX,
    pickable: false,
    colorTriggers: [colors],
  });
}

export function hoverColor(colors: DrawingColors): Rgba {
  return withOpacity(colors.focus, HOVER_OPACITY);
}
