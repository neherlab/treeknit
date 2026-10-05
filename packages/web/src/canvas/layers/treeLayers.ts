import type { Accessor, Color } from "@deck.gl/core";

import { type Rgba, withOpacity } from "../color";
import type { DrawingColors } from "../drawingColors";
import type { WorldPosition } from "../projection";
import { markLayer } from "./markLayer";
import { pathLayer } from "./pathLayer";

const BRANCH_WIDTH_PX = 1.5;

export const REASSORTMENT_WIDTH_PX = 2;

const MARK_RADIUS_PX = 3.5;

const MARK_LINE_PX = 1.5;

const SELECTION_RADIUS_PX = 7;

const SELECTION_LINE_PX = 2;

const HOVER_OPACITY = 0.35;

export interface BranchLayerOptions<D> {
  id: string;
  data: readonly D[];
  getPath: (object: D) => WorldPosition[];
  getColor: Accessor<D, Color>;
  colors: DrawingColors;
  widthPx?: number;
  dashed?: boolean;
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
  widthPx = BRANCH_WIDTH_PX,
  dashed = false,
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
    dashed,
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
  colorTriggers: readonly unknown[];
}

export function ringLayer<D>({
  id,
  data,
  getPosition,
  getLineColor,
  colors,
  colorTriggers,
}: RingLayerOptions<D>) {
  return markLayer({
    id,
    data,
    getPosition,
    getLineColor,
    getFillColor: colors.ground,
    radiusPx: MARK_RADIUS_PX,
    lineWidthPx: MARK_LINE_PX,
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
